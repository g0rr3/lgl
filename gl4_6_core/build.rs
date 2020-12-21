////////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2019 DasEtwas - All Rights Reserved                               /
//      Unauthorized copying of this file, via any medium is strictly prohibited   /
//      Proprietary and confidential                                               /
////////////////////////////////////////////////////////////////////////////////////

use gl_generator::{Api, DebugPrints, Fallbacks, Profile, Registry};
use std::io::{BufReader, Read};
use std::{env, fs::File, path::Path};
fn main() {}
fn a() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let bindingsdest = Path::new(&out_dir).join("bindings.rs");

    let mut file = File::create(&bindingsdest).expect("Could not create bindings file");

    #[cfg(feature = "fn_calls_print")]
    let print = DebugPrints::FunctionCalls;
    #[cfg(not(feature = "fn_calls_print"))]
    let print = DebugPrints::None;

    Registry::new(
        Api::Gl,
        (4, 6),
        Profile::Core,
        Fallbacks::All,
        [
            "GL_EXT_texture_filter_anisotropic",
            "GL_ARB_draw_buffers_blend",
            "GL_ARB_program_interface_query",
        ],
        print,
    )
    .write_bindings(gl_generator::GlobalTypedGenerator, &mut file)
    .unwrap();

    let cargodir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let libdest = Path::new(&cargodir).join("src").join("lib.rs");

    // check if new bindings are different from previous, if so, copy them into lib.rs
    if match std::fs::File::open(&libdest) {
        Ok(file) => {
            let mut content1 = Vec::new();
            BufReader::new(file).read_to_end(&mut content1).unwrap();
            let mut content2 = Vec::new();
            BufReader::new(File::open(&bindingsdest).expect("Could not open bindings file"))
                .read_to_end(&mut content2)
                .unwrap();
            content1
                .iter()
                .zip(content2.iter())
                .any(|(b1, b2)| b1 != b2)
        }
        _ => true,
    } {
        let _ = std::fs::remove_file(&libdest);
        std::fs::create_dir_all(&libdest.parent().unwrap())
            .expect("Failed to create dirs for lib.rs");
        std::fs::copy(bindingsdest, libdest).expect("Could not copy bindings into lib.rs");
    }
}
