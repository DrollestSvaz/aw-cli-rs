mod play;
mod search_input;
mod search_results;
mod episode_list;
mod post_play;

#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use std::process::{Command, Stdio};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use crossterm::cursor::MoveTo;
// use crossterm::style::Stylize;
#[cfg(target_os = "windows")]
use serde::{Deserialize, Serialize };
#[cfg(target_os = "windows")]
use colored::Colorize;
#[cfg(target_os = "windows")]
use dirs::config_dir;
#[cfg(target_os = "windows")]
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    vlc_path: Option<String>,
    fzf_installed: bool,
}
enum State {
    SearchInput,
    SearchResults(Option<String>),
    EpisodeList(String, String),
    Playing(String, usize, Vec<String>),
    PostPlay(String, usize, Vec<String>),
}

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    config_creator();


    let args: Vec<String> = std::env::args().collect();

    let initial_state = if args.contains(&"--news".to_string()) || args.contains(&"-n".to_string()) {
        State::SearchResults(Option::from(None))
    } else if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("usage: aw-cli-rs [-h] [--news] [-rc]");
        println!("Guarda anime dal terminale!");
        println!();
        println!("Informazioni:");
        println!("  -h, --help       mostra questo messaggio");
        println!();
        println!("Opzioni:");
        println!("  --news, -n           | mostra gli ultimi anime usciti su AnimeWorld");
        println!("  --remove-config, -rc | mostra gli ultimi anime usciti su AnimeWorld");
        std::process::exit(0);

    } else if args.contains(&"--remove-config".to_string()) || args.contains(&"-rc".to_string()){

        #[cfg(target_os = "windows")]
        {
            remove_config();
            std::process::exit(0);
        }
        #[cfg(not(target_os = "windows"))]
        {
            eprintln!("Comando disponibile solo su Windows.");
            std::process::exit(1);
        }

    } else if args.iter().skip(1).any(|a| a.starts_with('-')) {
        eprintln!("Argomento non riconosciuto. Usa --help per la lista dei comandi.");
        std::process::exit(1);
    } else {
        State::SearchInput
    };


    let mut state = initial_state;
    execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    loop {
        state = match state {
            State::SearchInput => search_input::search_input().await,
            State::SearchResults(query) => search_results::search_results(query).await,
            State::EpisodeList(anime, href) => episode_list::episode_list(anime, href).await,
            State::Playing(url, index, indirizzi) => play::play(url, index, indirizzi).await,
            State::PostPlay(url, index_ep , indirizzi) => post_play::post_play(url, index_ep, indirizzi).await,
        }
    }
}

#[cfg(target_os = "windows")]
fn config_creator()  {

    //Riutilizzo la variabile path per non sprecare memoria.
    if !dirs::config_dir().unwrap().join("aw-cli-rs").exists() {
        let mut vlc_path: String = String::new();
        let mut path: String = String::new();
        println!("{}", "Non è stato trovato il file di configurazione, vuoi procedere creandolo? (S/n)".red().bold());
        std::io::stdin().read_line(&mut path).unwrap();
        if path.to_lowercase().trim().eq("n") {
            std::process::exit(0);
        }

        println!("{}", "Hai già vlc installato? (s/N)".bold().on_white().truecolor(37, 37, 37));
        path.clear();
        std::io::stdin().read_line(&mut path).unwrap();
        if path.to_lowercase().trim().eq("s") {
            let mut control = false;
            println!("{}", "Inserire percorso assoluto di VLC (NON FARE CONFUSIONE CON IL PERCORSO DEL COLLEGAMENTO):");
            path.clear();
            std::io::stdin().read_line(&mut path).unwrap();
            path = path.trim().trim_matches('"').to_string();
            while !control {

                if Path::new(&path.trim()).exists() { control = true; vlc_path = path.trim().trim_matches('"').to_string(); }
                else {
                    println!("{}", "Path inserito non esistente!".red().bold());
                    println!("{}", "Reinserire...");
                    path.clear();
                    std::io::stdin().read_line(&mut path).unwrap();

                }
            }
        } else {
            println!("{}", "Permetteresti l'app di installarlo al posto tuo? (S/n)".on_white().bold().truecolor(37, 37, 37));
            path.clear();
            std::io::stdin().read_line(&mut path).unwrap();
            if path.to_lowercase().trim().eq("n") {
                println!("A presto. In attesa che scarichi VLC.");
                std::process::exit(0);
            } else {
                Command::new("winget").arg("install").arg("VideoLAN.VLC").spawn().unwrap().wait().unwrap();
                println!("{}", "Installazione terminata.");
                vlc_path = String::from(r"C:\Program Files\VideoLAN\VLC\vlc.exe");
            }

        }

        let fzf_installed = Command::new("where")
            .arg("fzf")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !fzf_installed {
            println!("{}", "Installazione di fzf, attendere...".bold());
            Command::new("winget").arg("install").arg("fzf").spawn().unwrap().wait().unwrap();
            println!("{}", "Installazione terminata.");
        }

        let config = Config {
            vlc_path: Option::from(vlc_path.trim().to_string()),
            fzf_installed: true,
        };

        let config_dir = dirs::config_dir().unwrap().join("aw-cli-rs");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_file = config_dir.join("config.toml");

        let toml = toml::to_string(&config).unwrap();
        std::fs::write(config_file, toml).unwrap();

    }
}

#[cfg(target_os = "windows")]
pub fn vlc_path() -> String {
    let content = std::fs::read_to_string(dirs::config_dir().unwrap().join(r"aw-cli-rs\config.toml")).unwrap();
    let config: Config = toml::from_str(&content).unwrap();
    config.vlc_path.unwrap()
}


#[cfg(not(target_os = "windows"))]
pub fn vlc_path() -> String {
    "vlc".to_string()
}

#[cfg(target_os = "windows")]
fn remove_config() {
    std::fs::remove_dir_all(dirs::config_dir().unwrap().join("aw-cli-rs")).unwrap();
}
