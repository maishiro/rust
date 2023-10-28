extern crate notify;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use std::{
    sync::mpsc::channel,
    thread,
    time::Duration,
};


fn main() {
    // Start file watch thread
    let (file_watch_tx, file_watch_rx) = channel();
    let watch_thread = thread::spawn(move || {
        let mut watcher: RecommendedWatcher = Watcher::new(file_watch_tx, Duration::from_millis(100)).unwrap();

        let watch_path = "./";
        match watcher.watch(watch_path, RecursiveMode::Recursive) {
            Ok(_) => println!("Watching path {}", watch_path),
            Err(e) => println!("Got an error trying to watch files! {:?}", e),
        }

        loop {
            match file_watch_rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Create(path) => println!("ファイルが作成されました: {:?}", path),
                        DebouncedEvent::Write(path) => println!("ファイルが書き込まれました: {:?}", path),
                        DebouncedEvent::Remove(path) => println!("ファイルが削除されました: {:?}", path),
                        DebouncedEvent::Rename(old_path, new_path) => println!("ファイルがリネームされました: {:?} -> {:?}", old_path, new_path),
                        DebouncedEvent::Rescan => println!("フォルダが再スキャンされました"),
                        DebouncedEvent::Error(err, path) => println!("エラーが発生しました: {:?}, {:?}", err, path),
                        _ => println!("その他のイベント: {:?}", event),
                    }
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    watch_thread.join().unwrap();    
}
