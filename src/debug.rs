pub struct Debugger {
    history: Vec<History>,
}

struct History {
    id: u16,
    last_str: String,
}

impl Debugger {
    pub fn new() -> Self {
        Self { history: vec![] }
    }

    pub fn print(&mut self, id: u16, str: String) {
        let opt_his_point = self.history.iter_mut().find(|his| his.id == id);

        if let Some(his_point) = opt_his_point {
            if &his_point.last_str != &str {
                println!("{}", &str);
                his_point.last_str = str;
            }
        } else {
            self.history.push(History { id, last_str: str });
        }
    }
}
