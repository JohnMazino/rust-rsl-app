use rusqlite::{Connection};
use std::path::PathBuf;

// НАХОДИМ ПУТЬ К БД
fn db_path() -> PathBuf {
    // получаем путь с помощью встроенного в dioxus JNI
    // https://stackoverflow.com/questions/79319500/how-can-i-get-the-path-of-android-internal-storage-in-dioxus0-6
    #[cfg(target_os = "android")]
    {
        match internal_storage_dir() {
            Ok(dir) => dir.join("slovo.db"),
            Err(e) => {
                eprintln!("⚠️ Не удалось получить путь, используем fallback: {}", e);
                PathBuf::from("/data/data/com.pisapopa.app/files/slovo.db")
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        let proj_dirs = directories::ProjectDirs::from("com", "pisapopa", "app")
            .expect("Не удалось получить директорию");
        proj_dirs.data_dir().join("slovo.db")
    }
}

#[cfg(target_os = "android")]
fn internal_storage_dir() -> anyhow::Result<PathBuf> {
    // Используем jni из dioxus::mobile::wry чтобы избежать конфликта версий
    use dioxus::mobile::wry::prelude::jni::objects::{JObject, JString};
    use dioxus::mobile::wry::prelude::jni::JNIEnv;

    let (tx, rx) = std::sync::mpsc::channel();

    dioxus::mobile::wry::prelude::dispatch(move |env: &mut JNIEnv, activity: &JObject, _webview: &JObject| {
        let result: anyhow::Result<PathBuf> = (|| {
            let files_dir = env
                .call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])?
                .l()?;

            let path_obj: JString = env
                .call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
                .l()?
                .into();

            let path: String = env.get_string(&path_obj)?.into();
            Ok(PathBuf::from(path))
        })();

        let _ = tx.send(result);
    });

    rx.recv().unwrap()
}

thread_local! {
    static DB: Connection = {
        let path = db_path();
        println!("📁 БД открыта по пути: {:?}", path);

        let conn = Connection::open(&path)
            .expect("Не удалось открыть базу данных");

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS gestures (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                video_filename TEXT NOT NULL,
                description TEXT,
                category TEXT
            );
            "#
        ).expect("Не удалось создать таблицу");

        conn
    };
}

pub fn with_db<F, R>(func: F) -> R
where
    F: FnOnce(&Connection) -> R,
{
    DB.with(func)
}

// Добавить несколько жестов
pub fn insert_default_gestures() {
    let defaults = vec![
        ("Victory", "victory.gif", "Победа / V знак", "basic"),
        ("Thumbs Up", "thumbs_up.gif", "Большой палец вверх", "basic"),
        ("Thumbs Down", "thumbs_down.gif", "Большой палец вниз", "basic"),
        ("Pointing", "pointing.gif", "Указательный палец", "basic"),
        ("OK", "ok.gif", "OK жест", "basic"),
    ];

    with_db(|db| {
        for (name, video, desc, cat) in defaults {
            let _ = db.execute(
                r#"
                INSERT OR IGNORE INTO gestures
                (name, video_filename, description, category)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                (name, video, desc, cat),
            );
        }
    });
}


// Обновление описания
pub fn update_gesture_description(id: i64, description: Option<String>) {
    with_db(|db| {
        let _ = db.execute(
            r#"
            UPDATE gestures
            SET description = ?1,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?2
            "#,
            (description, id),
        );
    });
}