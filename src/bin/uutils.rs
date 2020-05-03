/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Michael Gehring <mg@ebfe.org>
 *
 * For the full copyright and license information, please view the LICENSE
 * file that was distributed with this source code.
 */

// spell-checker:ignore (acronyms/names) Gehring
// spell-checker:ignore (rustlang) clippy concat rustlang
// spell-checker:ignore (uutils) coreutils uucore uumain uutils sigpipe
// spell-checker:ignore (shell) busybox

include!(concat!(env!("OUT_DIR"), "/uutils_crates.rs"));

extern crate lazy_static;
extern crate uucore;

use lazy_static::lazy_static;
use std::collections::hash_map::HashMap;
use std::io::Write;

static VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref BINARY_PATH: std::path::PathBuf = std::env::current_exe().unwrap();
    static ref NAME: &'static str = &*BINARY_PATH.file_stem().unwrap().to_str().unwrap();
}

include!(concat!(env!("OUT_DIR"), "/uutils_map.rs"));

fn usage(utils: &UtilityMap) {
    println!("{} {}", *NAME, VERSION);
    println!();
    println!("Usage:");
    println!("  {} [util [arguments...]]\n", *NAME);
    println!("Currently defined functions:");
    #[allow(clippy::map_clone)]
    let mut utils: Vec<&str> = utils.keys().map(|&s| s).collect();
    utils.sort();
    for util in utils {
        println!("\t{}", util);
    }
}

fn main() {
    uucore::panic::install_sigpipe_hook();

    let utils = util_map();
    let mut args: Vec<String> = uucore::args().collect();

    let binary = &BINARY_PATH;
    let binary_as_util = binary.file_stem().unwrap().to_str().unwrap();

    // binary name equals util name?
    if let Some(&uumain) = utils.get(binary_as_util) {
        std::process::exit(uumain(args));
    }

    // binary name equals prefixed util name?
    // * prefix/stem may be any string ending in a non-alphanumeric character
    if let Some(util) = utils.keys().find(|util| {
        binary_as_util.ends_with(*util)
            && !(&binary_as_util[..binary_as_util.len() - (*util).len()])
                .ends_with(char::is_alphanumeric)
    }) {
        // prefixed util => replace 0th (aka, executable name) argument
        args[0] = (*util).to_owned();
    } else {
        // unmatched binary name => regard as multi-binary container and advance argument list
        args.remove(0);
    }

    // 0th argument equals util name?
    if !args.is_empty() {
        let util = &args[0][..];

        match utils.get(util) {
            Some(&uumain) => {
                std::process::exit(uumain(args.clone()));
            }
            None => {
                if &args[0][..] == "--help" || &args[0][..] == "-h" {
                    // see if they want help on a specific util
                    if args.len() >= 2 {
                        let util = &args[1][..];
                        match utils.get(util) {
                            Some(&uumain) => {
                                let code = uumain(vec![util.to_owned(), "--help".to_owned()]);
                                std::io::stdout().flush().expect("could not flush stdout");
                                std::process::exit(code);
                            }
                            None => {
                                println!("{}: applet not found", util);
                                std::process::exit(1);
                            }
                        }
                    }
                    usage(&utils);
                    std::process::exit(0);
                } else {
                    println!("{}: applet not found", util);
                    std::process::exit(1);
                }
            }
        }
    } else {
        // no arguments provided
        usage(&utils);
        std::process::exit(0);
    }
}
