use clap::CommandFactory;
use clap_mangen;
use clap_mangen::Man;
use sway_scratch::Cli;

fn main() -> std::io::Result<()> {
    let man = Man::new(Cli::command());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let name = "sway-scratch";

    let out_dir =
        std::path::PathBuf::from(std::env::var_os("DESTDIR").ok_or(std::io::ErrorKind::NotFound)?);
    std::fs::write(out_dir.join(format!("{name}.1")), buffer)?;

    for subcommand in Cli::command().get_subcommands() {
        let subcommand_name = subcommand.get_name();
        let subcommand_name = format!("{name}-{subcommand_name}");
        let mut buffer: Vec<u8> = Default::default();
        let man = Man::new(subcommand.clone().name(&subcommand_name));
        man.render(&mut buffer)?;
        std::fs::write(
            std::path::PathBuf::from(&out_dir).join(format!("{}{}", &subcommand_name, ".1")),
            buffer,
        )?;
    }

    Ok(())
}
