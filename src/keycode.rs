use input_event_codes as k;

pub fn str_to_key(key: &str) -> u32 {
    match key {
        "a" => k::KEY_A!(),
        "b" => k::KEY_B!(),
        "c" => k::KEY_C!(),
        "d" => k::KEY_D!(),
        "e" => k::KEY_E!(),
        "f" => k::KEY_F!(),
        "g" => k::KEY_G!(),
        "h" => k::KEY_H!(),
        "i" => k::KEY_I!(),
        "j" => k::KEY_J!(),
        "k" => k::KEY_K!(),
        "l" => k::KEY_L!(),
        "m" => k::KEY_M!(),
        "n" => k::KEY_N!(),
        "o" => k::KEY_O!(),
        "p" => k::KEY_P!(),
        "q" => k::KEY_Q!(),
        "r" => k::KEY_R!(),
        "s" => k::KEY_S!(),
        "t" => k::KEY_T!(),
        "u" => k::KEY_U!(),
        "v" => k::KEY_V!(),
        "w" => k::KEY_W!(),
        "x" => k::KEY_X!(),
        "y" => k::KEY_Y!(),
        "z" => k::KEY_Z!(),
        "1" => k::KEY_1!(),
        "2" => k::KEY_2!(),
        "3" => k::KEY_3!(),
        "4" => k::KEY_4!(),
        "5" => k::KEY_5!(),
        "6" => k::KEY_6!(),
        "7" => k::KEY_7!(),
        "8" => k::KEY_8!(),
        "9" => k::KEY_9!(),
        "0" => k::KEY_0!(),
        "ENTER" => k::KEY_ENTER!(),
        "BACKSPACE" => k::KEY_BACKSPACE!(),
        "SPACE" => k::KEY_SPACE!(),
        _ => panic!("Unrecognised key"),
    }
}
