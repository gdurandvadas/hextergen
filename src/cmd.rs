use argh::FromArgs;
use hexx::orientation;

const TITLE: &str = r"

  ___ ___                   __                                     
 /   |   \   ____ ___  ____/  |_  ___________  ____   ____   ____  
/    ~    \_/ __ \\  \/  /\   __\/ __ \_  __ \/ ___\_/ __ \ /    \ 
\    Y    /\  ___/ >    <  |  | \  ___/|  | \/ /_/  >  ___/|   |  \
 \___|_  /  \___  >__/\_ \ |__|  \___  >__|  \___  / \___  >___|  /
       \/       \/      \/           \/     /_____/      \/     \/ 
____________________________________________________________________
";

#[derive(FromArgs, PartialEq)]
#[argh(description = "CLI for hextergen")]
struct Global {
    #[argh(subcommand)]
    nested: SubCommands,
}

#[derive(FromArgs, PartialEq)]
#[argh(subcommand)]
enum SubCommands {
    Generate(Generate),
}

#[derive(FromArgs, PartialEq)]
#[argh(subcommand, name = "generate", description = "Generate a new map")]
struct Generate {
    #[argh(
        option,
        short = 's',
        long = "seed",
        description = "seed from which to generate the map"
    )]
    seed: Option<u64>,

    #[argh(
        option,
        short = 'w',
        long = "width",
        description = "width of the map in hexes"
    )]
    width: Option<u32>,

    #[argh(
        option,
        short = 'h',
        long = "height",
        description = "height of the map in hexes"
    )]
    height: Option<u32>,

    #[argh(
        option,
        short = 'o',
        long = "orientation",
        description = "orientation of the hexes in the map (flat or pointy)"
    )]
    orientation: Option<String>,
}

#[derive(Debug)]
pub struct GenerateOptions {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub orientation: orientation::HexOrientation,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            seed: 0,
            width: 200,
            height: 160,
            orientation: orientation::HexOrientation::Flat,
        }
    }

}

pub fn run() {
    println!("{}", TITLE);

    let args: Global = argh::from_env();
    match args.nested {
        SubCommands::Generate(generate) => {
            let mut generate_options = GenerateOptions::default();
            if let Some(seed) = generate.seed {
                generate_options.seed = seed;
            }
            if let Some(width) = generate.width {
                generate_options.width = width;
            }
            if let Some(height) = generate.height {
                generate_options.height = height;
            }
            if let Some(orientation) = generate.orientation {
                match orientation.as_str() {
                    "flat" => generate_options.orientation = orientation::HexOrientation::Flat,
                    "pointy" => generate_options.orientation = orientation::HexOrientation::Pointy,
                    _ => {
                        eprintln!("Invalid orientation: {}", orientation);
                        std::process::exit(1);
                    }
                }
            }

            println!("Generating map with options: {:?}", generate_options);
        }
    }
}
