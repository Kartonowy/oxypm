use std::{
    io::{self, Read},
    process::{Child, Command, Stdio},
    time::SystemTime,
};
use splitty::split_unquoted_char;

const MAX_PROCESSES: usize = 10;
/* type ProcessPool = Arc<Mutex<Vec<Child>>>; */
type ProcessPool<'a> = Vec<Process<'a>>;

#[derive(Debug)]
struct Process<'a> {
    name: &'a str,
    command: Command,
    output: String,
    child: Child,
    started_at: SystemTime,
    finished: bool,
}

impl<'a> Process<'a> {
    fn new(name: &'a str, cli_command: &'a str, args: Vec<&'a str>) -> Process<'a> {
        let mut command = Command::new(cli_command);
        let child = command
            .stdout(Stdio::piped())
            .args(args)
            .spawn()
            .expect("Failed to execute command. Command probably not found, did you write it correctly?");
        Process {
            name,
            command,
            started_at: SystemTime::now(),
            child,
            output: String::with_capacity(8192),
            finished: false,
        }
    }

    fn invoke(&mut self) {
        assert!(self.command.get_program() != "");
        self.output = String::with_capacity(8192);
        self.child = self
            .command
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn a child in invoke");
        self.finished = false;
    }
    fn invoke_args(&mut self, args: Vec<&'a str>) {
        assert!(self.command.get_program() != "");
        self.output = String::with_capacity(8192);
        self.command.args(args);
        self.child = self
            .command
            .spawn()
            .expect("Failed to spawn a child in invoke");
        self.finished = false;
    }
}

fn add_process<'a>(process_pool: &mut ProcessPool<'a>, token: &'a str) {
    let (command, args) = parse_command(token);
    println!("{} {:?}", command, args);
    process_pool.push(Process::new(command, command, args));
}
/* Result<(&'a str, Vec<&'a str>), io::Error> */
fn parse_command<'a>(command: &'a str) -> (&'a str, Vec<&'a str>) {
    let splitted = command.splitn(2, " ").collect::<Vec<&str>>();
    (splitted[0], split_unquoted_char(splitted[1], ' ')
            .unwrap_quotes(true)
            .collect::<Vec<_>>())
}
pub fn init() {
    let mut process_pool: ProcessPool = Vec::with_capacity(MAX_PROCESSES);
    add_process(&mut process_pool, "sleep 2");
    add_process(&mut process_pool, "echo Hello");
    add_process(&mut process_pool, "echo World");
    add_process(&mut process_pool, "ls -al");

    loop {
        for process in &mut process_pool {
            if !process.finished {
                match process.child.try_wait() {
                    Ok(Some(_status)) => {
                        process
                            .child
                            .stdout
                            .as_mut()
                            .expect("Failed to get stdout ref")
                            .read_to_string(&mut process.output)
                            .expect("
                                Shouldn't happen, failed to unwrap value from stdout after finishing status.
                                output could be empty but should never panic.
                                ");
                        process.finished = true;
                        println!("{:?}", process);
                    }
                    Ok(None) => {}
                    Err(e) => println!("error attempting to wait {e}"),
                }
            }
        }

        if process_pool.len() <= 0 {
            break;
        }

        process_pool = process_pool.into_iter().filter(|e| !e.finished).collect();
    }
}
