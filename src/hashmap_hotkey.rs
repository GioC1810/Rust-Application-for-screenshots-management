use std::collections::HashMap;
use global_hotkey::hotkey::Code;
use global_hotkey::hotkey::Modifiers;

#[derive(Debug)]
pub struct HKdict {
    pub my_map: HashMap<String, Code>,
}

impl HKdict{

    pub fn new()->Self{
        let mut mappa= HashMap::new();
        mappa.insert("a".to_string(), Code::KeyA);
        mappa.insert("b".to_string(),  Code::KeyB);
        mappa.insert("c".to_string(),   Code::KeyC);
        mappa.insert("d".to_string(),   Code::KeyD);
        mappa.insert("e".to_string(),   Code::KeyE);
        mappa.insert("f".to_string(),   Code::KeyF);
        mappa.insert("g".to_string(),   Code::KeyG);
        mappa.insert("h".to_string(),   Code::KeyH);
        mappa.insert("i".to_string(),   Code::KeyI);
        mappa.insert("j".to_string(),   Code::KeyJ);
        mappa.insert("k".to_string(),   Code::KeyK);
        mappa.insert("l".to_string(),   Code::KeyL);
        mappa.insert("m".to_string(),   Code::KeyM);
        mappa.insert("n".to_string(),   Code::KeyN);
        mappa.insert("o".to_string(),   Code::KeyO);
        mappa.insert("p".to_string(),   Code::KeyP);
        mappa.insert("q".to_string(),   Code::KeyQ);
        mappa.insert("r".to_string(),   Code::KeyR);
        mappa.insert("s".to_string(),   Code::KeyS);
        mappa.insert("t".to_string(),   Code::KeyT);
        mappa.insert("u".to_string(),   Code::KeyU);
        mappa.insert("v".to_string(),   Code::KeyV);
        mappa.insert("w".to_string(),   Code::KeyW);
        mappa.insert("x".to_string(),   Code::KeyX);
        mappa.insert("y".to_string(),   Code::KeyY);
        mappa.insert("z".to_string(),   Code::KeyZ);

        return HKdict{ my_map : mappa};
    }
}



pub struct HKdictModifiers {
    pub my_map: HashMap<String, Modifiers>,
}

impl HKdictModifiers{

    pub fn new()->Self{
        let mut mappa= HashMap::new();
        mappa.insert("Shift".to_string(), Modifiers::SHIFT);
        mappa.insert("Control".to_string(),  Modifiers::CONTROL);
        mappa.insert("Alt".to_string(),   Modifiers::ALT);


        return HKdictModifiers{ my_map : mappa};
    }
}