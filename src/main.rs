use mem_snoop::process::*;
// use mem_snoop::temp::*;

fn main() {
    enum_processes()
        .unwrap()
        .into_iter()
        .for_each(|pid| match Process::open(pid) {
            Ok(proc) => match proc.name() {
                Ok(name) => println!("{}: {}", pid, name),
                Err(e) => println!("{}: (failed to get name: {})", pid, e),
            },
            Err(e) => eprintln!("failed to open {}: {}", pid, e),
        });
}
