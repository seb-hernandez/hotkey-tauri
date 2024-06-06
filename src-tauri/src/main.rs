// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Error, Result};
use core_foundation::array::CFIndex;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tauri::command]
async fn run() {
    let hotkeys = vec![
        "cmd+c".to_string(),
        "cmd+v".to_string(),
        "cmd+q".to_string(),
        "cmd+opt+esc".to_string(),
    ];
    let hotkeys_blocker_executor = Arc::new(RwLock::new(HotkeysBlockerExecutor::default()));
    let hotkeys_blocker_executor_ref = hotkeys_blocker_executor.clone();
    tokio::spawn(async move {
        hotkeys_blocker_executor_ref
            .read()
            .await
            .execute(hotkeys)
            .unwrap();
    });
    sleep(Duration::from_secs(10)).await;
    hotkeys_blocker_executor.read().await.stop();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub struct HotkeysBlockerExecutor {
    run_loop: CFRunLoop,
}

impl Default for HotkeysBlockerExecutor {
    fn default() -> Self {
        Self {
            run_loop: CFRunLoop::get_current(),
        }
    }
}

impl HotkeysBlockerExecutor {
    const RUNLOOP_SOURCE_ORDER: CFIndex = 0;
    const HOTKEYS_SPLITTER: char = '+';

    pub fn execute(&self, hotkeys: Vec<String>) -> Result<()> {
        let event_keys = HotkeysBlockerExecutor::convert_hotkeys_to_event_keys(&hotkeys);

        let event_callback_handler = |_, _, event: &CGEvent| {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let flags = event.get_flags();

            let block_hotkey = event_keys
                .iter()
                .any(|event_key| keycode == event_key.0 && flags.contains(event_key.1));

            if block_hotkey {
                event.set_type(CGEventType::Null);
            };

            Some(event.to_owned())
        };

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::KeyDown],
            event_callback_handler,
        )
        .map_err(|_| Error::msg("Failed to create event tap"))?;

        unsafe {
            let loop_source = tap
                .mach_port
                .create_runloop_source(Self::RUNLOOP_SOURCE_ORDER)
                .map_err(|_| Error::msg("Failed to create runloop source"))?;
            self.run_loop
                .add_source(&loop_source, kCFRunLoopCommonModes);
            tap.enable();
            CFRunLoop::run_current();
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.run_loop.stop();
    }

    fn convert_hotkeys_to_event_keys(hotkeys: &[String]) -> EventKeys {
        hotkeys
            .iter()
            .filter_map(|hotkey| {
                let (keycode, flags) = hotkey.split(Self::HOTKEYS_SPLITTER).fold(
                    (None, CGEventFlags::empty()),
                    |(keycode, flags), key| match EVENT_KEYS_MAP.get(key) {
                        None => (keycode, flags),
                        Some(EventKey::Int(i)) => (Some(*i), flags),
                        Some(EventKey::CGEventFlags(flag)) => (keycode, flags | *flag),
                    },
                );

                keycode.map(|keycode| (keycode, flags))
            })
            .collect()
    }
}

lazy_static! {
    pub static ref EVENT_KEYS_MAP: HashMap<&'static str, EventKey> = {
        let mut map = HashMap::new();

        map.insert("a", EventKey::Int(0));
        map.insert("s", EventKey::Int(1));
        map.insert("d", EventKey::Int(2));
        map.insert("f", EventKey::Int(3));
        map.insert("h", EventKey::Int(4));
        map.insert("g", EventKey::Int(5));
        map.insert("z", EventKey::Int(6));
        map.insert("x", EventKey::Int(7));
        map.insert("c", EventKey::Int(8));
        map.insert("v", EventKey::Int(9));
        map.insert("b", EventKey::Int(11));
        map.insert("q", EventKey::Int(12));
        map.insert("w", EventKey::Int(13));
        map.insert("e", EventKey::Int(14));
        map.insert("r", EventKey::Int(15));
        map.insert("y", EventKey::Int(16));
        map.insert("t", EventKey::Int(17));
        map.insert("o", EventKey::Int(31));
        map.insert("u", EventKey::Int(32));
        map.insert("i", EventKey::Int(34));
        map.insert("p", EventKey::Int(35));
        map.insert("l", EventKey::Int(37));
        map.insert("j", EventKey::Int(38));
        map.insert("k", EventKey::Int(40));
        map.insert("n", EventKey::Int(45));
        map.insert("m", EventKey::Int(46));
        map.insert("tab", EventKey::Int(48));
        map.insert("esc", EventKey::Int(53));

        map.insert(
            "cmd",
            EventKey::CGEventFlags(CGEventFlags::CGEventFlagCommand),
        );
        map.insert(
            "opt",
            EventKey::CGEventFlags(CGEventFlags::CGEventFlagAlternate),
        );
        map.insert(
            "shift",
            EventKey::CGEventFlags(CGEventFlags::CGEventFlagShift),
        );
        map.insert(
            "ctrl",
            EventKey::CGEventFlags(CGEventFlags::CGEventFlagControl),
        );

        map
    };
}

pub type EventKeys = Vec<(i64, CGEventFlags)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EventKey {
    Int(i64),
    CGEventFlags(CGEventFlags),
}

impl EventKey {
    fn as_int(&self) -> Option<i64> {
        match self {
            EventKey::Int(i) => Some(*i),
            _ => None,
        }
    }

    fn as_cg_event_flags(&self) -> CGEventFlags {
        match self {
            EventKey::CGEventFlags(flag) => *flag,
            _ => CGEventFlags::empty(),
        }
    }
}
