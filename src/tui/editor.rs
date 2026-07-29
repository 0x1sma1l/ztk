use std::{
    fmt,
    io::{Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, RwLock},
    thread,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{Child, CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use ratatui::{Frame, layout::Rect};
use tui_term::{vt100, widget::PseudoTerminal};

use crate::{cli::edit::EditBuffer, errors::AppError};

pub struct EditorSession {
    edit: Option<EditBuffer>,
    parser: Arc<RwLock<vt100::Parser>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    size: PtySize,
    dirty: Arc<AtomicBool>,
}

impl fmt::Debug for EditorSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorSession")
            .field("slug", &self.slug())
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

impl EditorSession {
    pub fn start(
        notes_dir: &std::path::Path,
        slug: &str,
        command: Vec<String>,
        cols: u16,
        rows: u16,
    ) -> Result<Self, AppError> {
        let edit = EditBuffer::prepare(notes_dir, slug)?;
        let size = PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = NativePtySystem::default()
            .openpty(size)
            .map_err(embedded_error)?;

        let mut arguments = command.into_iter();
        let executable = arguments.next().ok_or_else(|| {
            AppError::EmbeddedEditor("configured editor command is empty".to_string())
        })?;
        let mut builder = CommandBuilder::new(executable);
        if is_vim_family(builder.get_argv()[0].as_os_str()) {
            // The edit buffer is already temporary; swap files add no recovery value
            // and can fail in restricted system temporary directories.
            builder.arg("-n");
        }
        if is_neovim(builder.get_argv()[0].as_os_str()) {
            builder.env("NVIM_LOG_FILE", null_device());
        }
        builder.args(arguments);
        builder.arg(edit.path());
        builder.cwd(notes_dir);
        builder.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(builder).map_err(embedded_error)?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().map_err(embedded_error)?;
        let writer = pair.master.take_writer().map_err(embedded_error)?;
        let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows, size.cols, 0)));
        let output_parser = Arc::clone(&parser);
        let dirty = Arc::new(AtomicBool::new(true));
        let output_dirty = Arc::clone(&dirty);
        thread::Builder::new()
            .name("ztk-editor-output".to_string())
            .spawn(move || {
                let mut buffer = [0_u8; 8192];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            if let Ok(mut parser) = output_parser.write() {
                                parser.process(&buffer[..read]);
                                output_dirty.store(true, Ordering::Release);
                            }
                        }
                    }
                }
            })?;

        Ok(Self {
            edit: Some(edit),
            parser,
            writer,
            master: pair.master,
            child,
            size,
            dirty,
        })
    }

    pub fn slug(&self) -> &str {
        self.edit.as_ref().map(EditBuffer::slug).unwrap_or("")
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if let Ok(parser) = self.parser.read() {
            frame.render_widget(PseudoTerminal::new(parser.screen()), area);
        }
    }

    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::AcqRel)
    }

    pub fn send_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        if let Some(bytes) = encode_key(key) {
            self.writer.write_all(&bytes)?;
            self.writer.flush()?;
        }
        Ok(())
    }

    pub fn send_paste(&mut self, text: &str) -> Result<(), AppError> {
        self.writer.write_all(b"\x1b[200~")?;
        self.writer.write_all(text.as_bytes())?;
        self.writer.write_all(b"\x1b[201~")?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        let size = PtySize {
            rows: rows.max(1),
            cols: cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        };
        if size.rows == self.size.rows && size.cols == self.size.cols {
            return Ok(());
        }

        self.master.resize(size).map_err(embedded_error)?;
        if let Ok(mut parser) = self.parser.write() {
            parser.set_size(size.rows, size.cols);
        }
        self.size = size;
        self.dirty.store(true, Ordering::Release);
        Ok(())
    }

    pub fn has_exited(&mut self) -> Result<Option<bool>, AppError> {
        self.child
            .try_wait()
            .map(|status| status.map(|status| status.success()))
            .map_err(AppError::Io)
    }

    pub fn commit(mut self) -> Result<String, AppError> {
        let edit = self
            .edit
            .take()
            .ok_or_else(|| AppError::EmbeddedEditor("editor buffer is unavailable".to_string()))?;
        let slug = edit.slug().to_string();
        edit.commit()?;
        Ok(slug)
    }
}

impl Drop for EditorSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn embedded_error(error: impl fmt::Display) -> AppError {
    AppError::EmbeddedEditor(error.to_string())
}

fn is_vim_family(executable: &std::ffi::OsStr) -> bool {
    executable_name(executable).is_some_and(|name| matches!(name.as_str(), "vi" | "vim" | "nvim"))
}

fn is_neovim(executable: &std::ffi::OsStr) -> bool {
    executable_name(executable).is_some_and(|name| name == "nvim")
}

fn executable_name(executable: &std::ffi::OsStr) -> Option<String> {
    std::path::Path::new(executable)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
}

#[cfg(unix)]
fn null_device() -> &'static str {
    "/dev/null"
}

#[cfg(windows)]
fn null_device() -> &'static str {
    "NUL"
}

fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
    let modifiers = key.modifiers;
    let mut bytes = match key.code {
        KeyCode::Char(character) if modifiers.contains(KeyModifiers::CONTROL) => {
            let upper = character.to_ascii_uppercase() as u32;
            if (64..=95).contains(&upper) {
                vec![(upper as u8) & 0x1f]
            } else {
                character.to_string().into_bytes()
            }
        }
        KeyCode::Char(character) => character.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Up => modified_csi('A', modifiers),
        KeyCode::Down => modified_csi('B', modifiers),
        KeyCode::Right => modified_csi('C', modifiers),
        KeyCode::Left => modified_csi('D', modifiers),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::F(number) if (1..=4).contains(&number) => vec![0x1b, b'O', b'P' + number - 1],
        KeyCode::F(number) if (5..=12).contains(&number) => {
            let code = [15, 17, 18, 19, 20, 21, 23, 24][usize::from(number - 5)];
            format!("\x1b[{code}~").into_bytes()
        }
        _ => return None,
    };

    if modifiers.contains(KeyModifiers::ALT) && !bytes.starts_with(&[0x1b]) {
        bytes.insert(0, 0x1b);
    }
    Some(bytes)
}

fn modified_csi(final_byte: char, modifiers: KeyModifiers) -> Vec<u8> {
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let alt = modifiers.contains(KeyModifiers::ALT);
    let control = modifiers.contains(KeyModifiers::CONTROL);
    let parameter = 1 + u8::from(shift) + 2 * u8::from(alt) + 4 * u8::from(control);
    if parameter == 1 {
        format!("\x1b[{final_byte}").into_bytes()
    } else {
        format!("\x1b[1;{parameter}{final_byte}").into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{encode_key, is_vim_family};

    #[test]
    fn editor_key_encoding_supports_text_control_and_navigation() {
        assert_eq!(
            encode_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            Some(b"x".to_vec())
        );
        assert_eq!(
            encode_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(vec![3])
        );
        assert_eq!(
            encode_key(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL)),
            Some(b"\x1b[1;5A".to_vec())
        );
    }

    #[test]
    fn vim_family_detection_supports_paths_and_ignores_gui_editors() {
        assert!(is_vim_family(std::ffi::OsStr::new("/usr/local/bin/nvim")));
        assert!(is_vim_family(std::ffi::OsStr::new("vim")));
        assert!(!is_vim_family(std::ffi::OsStr::new("code")));
    }
}
