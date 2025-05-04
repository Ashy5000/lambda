use std::sync::{Arc, Mutex};
use std::thread;
use soloud::*;

fn init_sl() -> Soloud {
    Soloud::default().unwrap()
}

pub(crate) fn sound_thread() -> Arc<Mutex<bool>> {
    let trigger_flag = Arc::new(Mutex::new(false));

    let trigger = Arc::clone(&trigger_flag);
    thread::spawn(move || {
        let sl = init_sl();
        sl.stop_all();
        let mut wav0 = Wav::default();
        wav0.load_mem(include_bytes!("../reducing.mp3")).unwrap();
        let mut wav1 = Wav::default();
        wav1.load_mem(include_bytes!("../normal_form.mp3")).unwrap();
        wav1.set_volume(0.3);

        loop {
            sl.stop_all();
            while !*trigger.lock().unwrap() {
                thread::sleep(std::time::Duration::from_millis(100));
            }
            *trigger.lock().unwrap() = false;
            sl.stop_all();
            sl.play(&wav0);
            while !*trigger.lock().unwrap() {
                thread::sleep(std::time::Duration::from_millis(100));
                if sl.voice_count() == 0 {
                    sl.play(&wav0);
                }
            }
            *trigger.lock().unwrap() = false;
            sl.stop_all();
            sl.play(&wav1);
            while sl.voice_count() > 0 {
                thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    });

    trigger_flag
}