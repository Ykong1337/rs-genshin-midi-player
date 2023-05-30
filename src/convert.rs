use crate::midi::Midi;
use rfd::MessageButtons;
use std::fs::File;
use std::io::Write;
use std::thread;

pub fn convert_from_midi(file_name: String, midi: Midi) {
    let _ = thread::spawn(move || {
        let mut key = File::create(format!("{}.txt", file_name.to_string())).unwrap();
        let mut key_phone = File::create(format!("phone-{}.txt", file_name)).unwrap();
        let mut res = String::new();
        for e in midi.events.lock().unwrap().iter() {
            match e.press {
                24 => res.push('z'),
                26 => res.push('x'),
                28 => res.push('c'),
                29 => res.push('v'),
                31 => res.push('b'),
                33 => res.push('n'),
                35 => res.push('m'),
                36 => res.push('z'),
                38 => res.push('x'),
                40 => res.push('c'),
                41 => res.push('v'),
                43 => res.push('b'),
                45 => res.push('n'),
                47 => res.push('m'),
                48 => res.push('z'),
                50 => res.push('x'),
                52 => res.push('c'),
                53 => res.push('v'),
                55 => res.push('b'),
                57 => res.push('n'),
                59 => res.push('m'),
                60 => res.push('a'),
                62 => res.push('s'),
                64 => res.push('d'),
                65 => res.push('f'),
                67 => res.push('g'),
                69 => res.push('h'),
                71 => res.push('j'),
                72 => res.push('q'),
                74 => res.push('w'),
                76 => res.push('e'),
                77 => res.push('r'),
                79 => res.push('t'),
                81 => res.push('y'),
                83 => res.push('u'),
                84 => res.push('q'),
                86 => res.push('w'),
                88 => res.push('e'),
                89 => res.push('r'),
                91 => res.push('t'),
                93 => res.push('y'),
                95 => res.push('u'),
                _ => {}
            }
            match e.delay {
                e if e > 50.0 && e <= 700.0 => res.push_str(" - "),
                e if e > 700.0 && e <= 2000.0 => res.push_str(" / "),
                e if e > 2000.0 => res.push_str("\n\n"),
                _ => {}
            }
        }
        let phone = res
            .replace("q", "+1")
            .replace("w", "+2")
            .replace("e", "+3")
            .replace("r", "+4")
            .replace("t", "+5")
            .replace("y", "+6")
            .replace("u", "+7")
            .replace("a", "1")
            .replace("s", "2")
            .replace("d", "3")
            .replace("f", "4")
            .replace("g", "5")
            .replace("h", "6")
            .replace("j", "7")
            .replace("z", "-1")
            .replace("x", "-2")
            .replace("c", "-3")
            .replace("v", "-4")
            .replace("b", "-5")
            .replace("n", "-6")
            .replace("m", "-7");
        key.write(res.to_uppercase().as_bytes()).unwrap();
        key_phone.write(phone.as_bytes()).unwrap();
        rfd::MessageDialog::new()
            .set_description("转换成功\n请查看当前目录下的txt文本文件")
            .set_buttons(MessageButtons::Ok)
            .show();
    });
}
