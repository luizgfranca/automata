use std::{io::Read, process::{Child, Command, Stdio}};

pub fn try_run(cmd: &Vec<String>) {
    if let Some(app) = cmd.get(0) {
        let mut command = Command::new(app);
        let args = &cmd[1..];
        for it in args {
            command.arg(&it);
        }

        let output = command.spawn();
        match output {
            Err(e) => println!("unable to spawn process {}", e.to_string()),
            _ => (),
        }
    }
}

pub fn find(root: &str, name: &str) -> String {
    let mut command = Command::new("find");
    let name_query = format!("*{}*", name);
    command.args(vec![root, "-name", &name_query]);
    dbg!(&command);

    let child = command.stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    let output = child
        .wait_with_output()
        .expect("failed to wait on child");

    let data = output.stdout.to_vec();

    let out = String::from_utf8(data.to_vec()).expect("unable to interpret output as string");
    out
}
