use std::fs;
use std::path::{Path, PathBuf};

pub struct FileExplorer {
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
}

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_conf: bool,
}

impl FileExplorer {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut explorer = Self {
            current_dir: home,
            entries: vec![],
            selected: 0,
            scroll_offset: 0,
        };
        explorer.reload();
        explorer
    }

    pub fn reload(&mut self) {
        self.entries.clear();
        self.selected = 0;
        self.scroll_offset = 0;

        // Добавляем ".." для перехода наверх
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                is_conf: false,
            });
        }

        // Читаем содержимое директории
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut dirs = vec![];
            let mut files = vec![];

            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Скрываем скрытые файлы (начинающиеся с '.')
                if name.starts_with('.') { continue; }

                let is_dir = path.is_dir();
                let is_conf = path.extension()
                    .map(|e| e == "conf")
                    .unwrap_or(false);

                let entry = DirEntry { name, path, is_dir, is_conf };

                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }

            // Сортируем: директории вверху, потом файлы
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));

            self.entries.extend(dirs);
            self.entries.extend(files);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self, visible_height: usize) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
            if self.selected >= self.scroll_offset + visible_height {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn enter(&mut self) -> Option<PathBuf> {
        let entry = &self.entries[self.selected];
        if entry.is_dir {
            self.current_dir = entry.path.clone();
            self.reload();
            None
        } else if entry.is_conf {
            Some(entry.path.clone())
        } else {
            None
        }
    }

    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }
}