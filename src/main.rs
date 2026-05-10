mod bd;
use bd::*;
mod components;
use dioxus::prelude::*;
use std::collections::HashMap;
use dioxus::desktop::Config;
use dioxus_primitives::accordion::*;
use manganis::AssetOptions;

#[derive(Clone, PartialEq)]
struct Gesture {
    id: i64,
    name: String,
    video_filename: String,
    description: Option<String>,
    category: Option<String>
}

const VISION_BUNDLE: Asset = asset!("/assets/mediapipe/wasm/vision_bundle.mjs");
const VISION_WASM: Asset = asset!("/assets/mediapipe/wasm");
const GESTURE_MODEL: Asset = asset!("/assets/mediapipe/gesture_recognizer.task");

const ICON_HOME: Asset = asset!("/assets/icon/home.png", AssetOptions::image().with_webp());
const ICON_VIDEO: Asset = asset!("/assets/icon/camera.png", AssetOptions::image().with_webp());
const ICON_SETTINGS: Asset = asset!("/assets/icon/settings.png", AssetOptions::image().with_webp());
const ICON_ARROW: Asset = asset!("/assets/icon/arrow.png", AssetOptions::image().with_webp());
const ICON_EXIT: Asset = asset!("/assets/icon/exit.png", AssetOptions::image().with_webp());
const PAPER: Asset = asset!("/public/paper.png", AssetOptions::image().with_webp());
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const ACCORDING_CSS: Asset = asset!("/src/components/accordion/style.css");
const FONT: Asset = asset!("/assets/fonts/hi-melody-cyrillic.otf");
const GIFS: Asset = asset!("/assets/gifs", AssetOptions::builder().with_hash_suffix(false));

fn main() {
    insert_default_gestures();

    LaunchBuilder::desktop().with_cfg(
        Config::new().with_custom_head(
            format!(
                r#"
                    <style>
                        @font-face {{
                            font-family: "Molli";
                            font-weight: normal;
                            font-style: normal;
                            src: url("{0}") format("truetype");
                        }}
                        * {{
                            font-family: "Molli", sans-serif !important;
                        }}
                    </style>
                    "#,
                FONT.to_string()
            )
        )
    ).launch(App);
}

#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[layout(Header)]
    #[route("/")]
    Home,
    #[route("/video")]
    Video,
    #[route("/settings")]
    Settings,
    #[route("/translate")]
    Translate,
    #[route("/gesture/:id")]
    GestureDetail { id: i64 },
}


#[component]
fn App() -> Element {
    rsx!(
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: ACCORDING_CSS }

        Router::<Route> {}
    )
}

#[component]
fn Header() -> Element {

    rsx! {
        nav {
            div { class: "max-w-lg mx-auto flex items-center justify-around h-16 px-4",

                // Главная
                Link {
                    to: Route::Home {},
                    class: "flex flex-col items-center justify-center flex-1 py-1 text-white/80 hover:text-emerald-400 transition-all duration-300 relative group",
                    img {
                        src: ICON_HOME,
                        loading: "lazy",
                        class: "w-12 h-12 transition-transform group-active:scale-110",
                        alt: "Главная"
                    }
                    /*span { class: "text-xs font-medium tracking-wider mt-1", "Главная" }*/
                }

                // Видео
                Link {
                    to: Route::Video {},
                    class: "flex flex-col items-center justify-center flex-1 py-1 text-white/80 hover:text-emerald-400 transition-all duration-300 relative group",
                    img {
                        src: ICON_VIDEO,
                        loading: "lazy",
                        class: "w-15 h-15 transition-transform group-active:scale-110",
                        alt: "Видео"
                    }
                    /*span { class: "text-xs font-medium tracking-wider mt-1", "Видео" }*/
                }
                // Переводчик
                Link {
                    to: Route::Translate {},
                    class: "flex flex-col items-center justify-center flex-1 py-1 text-white/80 hover:text-emerald-400 transition-all duration-300 relative group",
                    img {
                        src: ICON_SETTINGS,
                        loading: "lazy",
                        class: "w-11 h-11 transition-transform group-active:scale-110",
                        alt: "Переводчик"
                    }
                    /*span { class: "text-xs font-medium tracking-wider mt-1", "Переводчик" }*/
                }

                // Настройки
                Link {
                    to: Route::Settings {},
                    class: "flex flex-col items-center justify-center flex-1 py-1 text-white/80 hover:text-emerald-400 transition-all duration-300 relative group",
                    img {
                        src: ICON_SETTINGS,
                        loading: "lazy",
                        class: "w-11 h-11 transition-transform group-active:scale-110",
                        alt: "Настройки"
                    }
                    /*span { class: "text-xs font-medium tracking-wider mt-1", "Настройки" }*/
                }

            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn CategoryAccordion(category: String, gestures: Vec<Gesture>) -> Element {
    let mut is_open = use_signal(|| false);

    rsx! {
        div {
            class: "accordion-item",
            "data-open": "{is_open()}",

            div {
                class: "accordion-trigger",
                onclick: move |_| is_open.toggle(),
                span { "{category}" }
                div {
                    class: "accordion-expand-icon",
                    img {
                        src: ICON_ARROW,
                        loading: "lazy",
                        width: "30"
                    }
                }
            }

            div {
                class: "accordion-content",
                "data-open": "{is_open()}",
                div { class: "accordion-inner",
                    for gesture in gestures {
                        Link {
                            to: Route::GestureDetail { id: gesture.id },
                            div { class: "gesture-item transition-all",
                                "{gesture.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let mut gestures = use_signal(Vec::<Gesture>::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);

            let data = with_db(|db| {
                let mut stmt = db.prepare(
                    r#"
                    SELECT id, name, video_filename, description, category
                    FROM gestures
                    ORDER BY category ASC, name ASC
                    "#
                ).unwrap();

                stmt.query_map([], |row| {
                    Ok(Gesture {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        video_filename: row.get(2)?,
                        description: row.get(3)?,
                        category: row.get(4)?
                    })
                })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>()
            });

            gestures.set(data);
            loading.set(false);
        });
    });

    let grouped_gestures = use_memo(move || {
        let mut map: HashMap<String, Vec<Gesture>> = HashMap::new();
        for gesture in gestures.read().iter() {
            let category = gesture.category
                .clone()
                .unwrap_or_else(|| "Без категории".to_string());
            map.entry(category).or_default().push(gesture.clone());
        }
        let mut groups: Vec<(String, Vec<Gesture>)> = map.into_iter().collect();
        groups.sort_by_key(|(cat, _)| cat.clone());
        groups
    });

    rsx! {
        div {class: "p-6 max-w-6xl mx-auto min-h-screen pb-24",
            for (category, group) in grouped_gestures.read().iter() {
                CategoryAccordion {
                    category: category.clone(),
                    gestures: group.clone()
                }
                CategoryAccordion {
                    category: category.clone(),
                    gestures: group.clone()
                }
            }
        }
    }
}

#[component]
fn Video() -> Element {
    let mut stream_active = use_signal(|| false);

    let toggle_camera = move |_| {
        if stream_active() {
            let _ = document::eval(r#"
                if (window.currentStream) { window.currentStream.getTracks().forEach(t => t.stop()); window.currentStream = null; }
                const c = document.getElementById('canvas'); if (c) c.style.display = "none";
            "#);
            stream_active.set(false);
        } else {
            let bundle_url = VISION_BUNDLE.to_string();
            let wasm_base = VISION_WASM.to_string();
            let model_url = GESTURE_MODEL.to_string();

            let js_code = format!(r##"
                async function start() {{
                    const canvas = document.getElementById('canvas');
                    const ctx = canvas.getContext('2d', {{ alpha: false }});

                    const bundleFull = new URL('{0}', window.location.href).href;
                    const wasmFull = new URL('{1}', window.location.href).href;
                    const modelFull = new URL('{2}', window.location.href).href;

                    const {{ FilesetResolver, GestureRecognizer }} = await import(bundleFull);

                    const vision = await FilesetResolver.forVisionTasks(wasmFull);

                    window.gestureRecognizer = await GestureRecognizer.createFromOptions(vision, {{
                        baseOptions: {{
                            modelAssetPath: modelFull,
                            delegate: "GPU"
                        }},
                        runningMode: "VIDEO",
                        numHands: 2,
                        minHandDetectionConfidence: 0.45,
                        minHandPresenceConfidence: 0.45,
                        minHandTrackingConfidence: 0.45
                    }});

                    const stream = await navigator.mediaDevices.getUserMedia({{
                        video: {{
                            facingMode: "user",
                            width: {{ ideal: 480 }},
                            height: {{ ideal: 360 }}
                        }}
                    }});

                    window.currentStream = stream;

                    const video = document.createElement('video');
                    video.style.display = "none";
                    video.autoplay = true;
                    video.playsInline = true;
                    video.srcObject = stream;
                    document.body.appendChild(video);
                    await video.play();

                    window.hiddenVideo = video;

                    function resizeCanvas() {{
                        const aspect = video.videoWidth / video.videoHeight;
                        canvas.width = 480;
                        canvas.height = Math.round(480 / aspect);
                        canvas.style.width = "100%";
                        canvas.style.maxWidth = "480px";
                        canvas.style.height = "auto";
                    }}

                    video.onloadedmetadata = resizeCanvas;
                    resizeCanvas();

                    canvas.style.display = "block";

                    let lastDetection = 0;
                    const TARGET_FPS = 60;

                    function loop() {{
                        const now = Date.now();

                        if (now - lastDetection > 1000 / TARGET_FPS) {{
                            lastDetection = now;

                            const results = window.gestureRecognizer.recognizeForVideo(video, now);

                            // Отрисовка видео (зеркально)
                            ctx.save();
                            ctx.scale(-1, 1);
                            ctx.drawImage(video, -canvas.width, 0, canvas.width, canvas.height);
                            ctx.restore();

                            let gestureText = " ";

                            if (results.gestures && results.gestures.length > 0) {{
                                const topGesture = results.gestures[0][0]; // лучший жест первой руки
                                gestureText = topGesture.categoryName + " (" + (topGesture.score * 100).toFixed(0) + "%)";

                                // Рисуем landmarks (для наглядности)
                                if (results.landmarks) {{
                                    ctx.strokeStyle = "#22ff88";
                                    ctx.lineWidth = 3;
                                    ctx.fillStyle = "#ff2266";

                                    for (const lm of results.landmarks) {{
                                        const fingers = [[0,1,2,3,4],[0,5,6,7,8],[0,9,10,11,12],[0,13,14,15,16],[0,17,18,19,20]];
                                        for (const f of fingers) {{
                                            ctx.beginPath();
                                            for (let i = 0; i < f.length; i++) {{
                                                const p = lm[f[i]];
                                                const x = (1 - p.x) * canvas.width;
                                                const y = p.y * canvas.height;
                                                i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
                                            }}
                                            ctx.stroke();
                                        }}

                                        for (const p of lm) {{
                                            const x = (1 - p.x) * canvas.width;
                                            const y = p.y * canvas.height;
                                            ctx.beginPath();
                                            ctx.arc(x, y, 4, 0, Math.PI * 2);
                                            ctx.fill();
                                        }}
                                    }}
                                }}
                            }}

                            ctx.save();
                            ctx.fillStyle = "#22ff88";
                            ctx.font = "bold 28px Arial";
                            ctx.textAlign = "center";
                            ctx.fillText(gestureText, canvas.width / 2, 60);
                            ctx.restore();
                        }} else {{
                            ctx.save();
                            ctx.scale(-1, 1);
                            ctx.drawImage(video, -canvas.width, 0, canvas.width, canvas.height);
                            ctx.restore();
                        }}

                        requestAnimationFrame(loop);
                    }}

                    loop();
                }}

                start().catch(e => alert("GestureRecognizer ошибка: " + e.message));
            "##, bundle_url, wasm_base, model_url);

            let _ = document::eval(&js_code);
            stream_active.set(true);
        }
    };

    rsx! {
        div { class: "flex flex-col items-center p-4",
            button {
                class: "camera_button",
                onclick: toggle_camera,
                if stream_active() {
                    "Выключить камеру"
                } else {
                    "Включить камеру"
                }
            }

            canvas {
                id: "canvas"
            }
        }
    }
}

#[component]
fn Settings() -> Element {
    let mut gestures = use_signal(Vec::<Gesture>::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            loading.set(true);

            let data = with_db(|db| {
                let mut stmt = db.prepare(
                    r#"
                    SELECT id, name, video_filename, description, category
                    FROM gestures
                    ORDER BY name ASC
                    "#
                ).unwrap();

                stmt.query_map([], |row| {
                    Ok(Gesture {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        video_filename: row.get(2)?,
                        description: row.get(3)?,
                        category: row.get(4)?
                    })
                })
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>()
            });

            gestures.set(data);
            loading.set(false);
        });
    });

    rsx! {
        div { class: "p-6 max-w-6xl mx-auto",
            div { class: "flex justify-between items-center mb-8",
                h1 { class: "text-3xl font-bold text-white", "База жестов" }
                span { class: "px-4 py-2 bg-emerald-900/70 text-emerald-400 text-sm rounded-2xl font-medium",
                    "{gestures.len()} записей"
                }
            }

            if loading() {
                div { class: "flex justify-center py-20", "Загрузка базы данных..." }
            } else if gestures.is_empty() {
                div { class: "text-center py-20 text-gray-400",
                    "В базе пока нет жестов"
                }
            } else {
                // ← Главное изменение здесь
                div { class: "overflow-x-auto pb-4 -mx-6 px-6", // -mx-6 + px-6 убирает боковые отступы для удобного скролла
                    div { class: "bg-zinc-900/95 backdrop-blur-2xl border border-white/10 rounded-3xl overflow-hidden min-w-[720px]",
                        table { class: "w-full min-w-[720px] border-collapse",
                            thead {
                                tr { class: "bg-zinc-950 border-b border-white/10",
                                    th { class: "px-6 py-5 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider",
                                        "Название жеста"
                                    }
                                    th { class: "px-6 py-5 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider",
                                        "Видео файл"
                                    }
                                    th { class: "px-6 py-5 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider",
                                        "Описание"
                                    }
                                    th { class: "px-6 py-5 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider",
                                        "Категория"
                                    }
                                }
                            }
                            tbody { class: "divide-y divide-white/10 text-sm",
                                for gesture in gestures.iter() {
                                    tr { class: "hover:bg-white/5 transition-colors",
                                        td { class: "px-6 py-4 font-medium text-white whitespace-nowrap",
                                            "{gesture.name}"
                                        }
                                        td { class: "px-6 py-4 font-mono text-gray-400 text-sm whitespace-nowrap",
                                            "{gesture.video_filename}"
                                        }
                                        td { class: "px-6 py-4 text-gray-300",
                                            if let Some(desc) = &gesture.description {
                                                "{desc}"
                                            } else {
                                                span { class: "text-gray-500 italic",
                                                    "—"
                                                }
                                            }
                                        }
                                        td { class: "px-6 py-4",
                                            span { class: "inline-flex px-3 py-1 text-xs font-medium rounded-full bg-sky-500/10 text-sky-400 whitespace-nowrap",
                                                {gesture.category.as_deref().unwrap_or("без категории")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Кнопка тестовых данных
            div { class: "mt-10 flex justify-center",
                button {
                    class: "px-8 py-4 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 rounded-2xl font-semibold text-white shadow-lg active:scale-95 transition-all flex items-center gap-3",
                    onclick: move |_| {
                        spawn(async {
                            insert_default_gestures();
                            // После нажатия таблица обновится автоматически (effect перезапустится при следующем рендере)
                        });
                    },
                    "Заполнить тестовыми жестами"
                }
            }
        }
    }
}

#[component]
fn Translate() -> Element {
    rsx! {
        Accordion { allow_multiple_open: false, horizontal: false,
            for i in 0..4 {
                AccordionItem { index: i,
                    AccordionTrigger { "the quick brown fox" }
                    AccordionContent {
                        div { padding_bottom: "1rem",
                            p { padding: "0",
                                "lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum lorem ipsum"
                            }
                        }
                    }
                }
            }
        }
        div {h1 { "ТУТ "}
            img {
                src: PAPER,
                loading: "lazy",
            }
        }
    }
}

#[component]
fn GestureDetail(id: i64) -> Element {
    let mut gesture = use_signal(|| None::<Gesture>);
    let mut loading = use_signal(|| true);
    let mut editing_desc = use_signal(String::new);
    let mut is_saving = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let found = with_db(|db| {
                db.prepare(
                    r#"
                    SELECT id, name, video_filename, description, category
                    FROM gestures
                    WHERE id = ?
                    "#
                ).unwrap()
                    .query_row([id], |row| {
                        Ok(Gesture {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            video_filename: row.get(2)?,
                            description: row.get(3)?,
                            category: row.get(4)?
                        })
                    }).ok()
            });

            if let Some(g) = &found {
                editing_desc.set(g.description.clone().unwrap_or_default());
            }
            gesture.set(found);
            loading.set(false);
        });
    });

    // Функция сохранения
    let save_description = move |_| {
        let new_desc = editing_desc();

        is_saving.set(true);

        spawn(async move {
            let success = with_db(|db| {
                db.prepare(
                    "UPDATE gestures SET description = ? WHERE id = ?"
                ).unwrap()
                    .execute((new_desc.clone(), id))
                    .is_ok()
            });

            if success {
                // Обновляем локальное состояние
                gesture.set(gesture().map(|mut g| {
                    g.description = if new_desc.is_empty() { None } else { Some(new_desc.clone()) };
                    g
                }));
            }

            is_saving.set(false);
        });
    };

    rsx! {
        div {
            Link {
                to: Route::Home {},
                class: "back",
                img {
                    src: ICON_EXIT,
                    loading: "lazy",
                    class: "w-12 h-12",
                }
            }
            if let Some(g) = gesture() {
                    div {{
                        let asset_path = format!("{}/{}", GIFS, g.video_filename);
                        rsx! {
                        img {
                            src: "{asset_path}",
                            loading: "lazy",
                            class: "gif_gesture",
                            alt: "{g.name}",
                            }
                        }
                    }}
                    // Редактируемое описание
                    div {
                        textarea {
                            value: "{editing_desc}",
                            oninput: move |e| editing_desc.set(e.value()),
                            placeholder: "{editing_desc}"
                        }

                        div {
                            button {
                                class: "save_description_button",
                                onclick: save_description,
                                "Сохранить описание"
                            }
                        }
                    }
            }
        }
    }
}