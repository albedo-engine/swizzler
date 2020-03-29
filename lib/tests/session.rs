use image::{open, Rgba};
use regex::Regex;
use swizzler::session::{
    resolve_assets_dir,
    GenericAssetReader,
    GenericTarget,
    RegexMatcher,
    Session,
};

fn start_session() {
    let resolver = GenericAssetReader::new()
    .set_base(Regex::new(r"(.*)_.*").unwrap())
    .add_matcher(
        Box::new(RegexMatcher::new("albedo", Regex::new(r"(?i)albedo").unwrap()))
    )
    .add_matcher(
        Box::new(RegexMatcher::new("ao", Regex::new(r"(?i)ao").unwrap()))
    );

    let albedo_ao_target = GenericTarget::new(vec![
        Some(("albedo", 0)), Some(("albedo", 1)), Some(("albedo", 2)), Some(("ao", 0))
    ]).set_name(String::from("_albedo-ao.png"));

    let session = Session::new()
        .add_target(albedo_ao_target)
        .set_output_folder(std::path::PathBuf::from("./_tests_output_"));

    let folder = std::path::PathBuf::from("./tests/textures");
    let assets = resolve_assets_dir(&folder, &resolver);
    assert!(assets.is_ok(), "assets folder should be processed with no error");

    let errors = session.run(&assets.unwrap());
    for e in &errors {
        eprintln!("Error processing file: {:?}", e);
    }
    assert_eq!(errors.len(), 0, "errors list should be empty");
}

fn test_image(
    path: &str,
    expected_dimensions: (u32, u32),
    expected: &[ Rgba<u8> ]
) {
    let img = open(path).unwrap();
    let img = img.as_rgba8().unwrap();
    assert_eq!(
        img.dimensions(),
        expected_dimensions,
        "invalid dimension for image `{}`", path
    );
    let (width, _) = img.dimensions();
    for (i, px_exp) in expected.iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        let px = img.get_pixel(x, y);
        for j in 0..4 {
            let channel: i16 = px[j].into();
            let channel_exp: i16 = px_exp[j].into();
            assert!(
                (channel - channel_exp).abs() <= 1,
                "pixel ({}, {}) mismatch at channel `{}`, expected `{}`, got `{}`. Texture: `{}`. ",
                x, y, j, channel_exp, channel, path
            );
        }
    }
}

#[test]
fn run_session() {
    start_session();
    // Test that each file exists.
    assert!(
        std::fs::metadata("./_tests_output_/a_albedo-ao.png").is_ok(),
        "`a_albedo-ao.png` should be created"
    );
    assert!(
        std::fs::metadata("./_tests_output_/b_albedo-ao.png").is_ok(),
        "`b_albedo-ao.png` should be created"
    );
    assert!(
        std::fs::metadata("./_tests_output_/rec/a_albedo-ao.png").is_ok(),
        "`rec/a_albedo-ao.png` should be created"
    );
}

#[test]
fn run_session_check_texture() {
    start_session();

    test_image("./_tests_output_/a_albedo-ao.png", (2, 2), &[
        Rgba([ 0, 0, 255, 0 ]),
        Rgba([ 0, 255, 0, 127 ]),
        Rgba([ 255, 0, 0, 255 ]),
        Rgba([ 127, 127, 127, 0 ])
    ]);

    test_image("./_tests_output_/b_albedo-ao.png", (2, 2), &[
        Rgba([ 255, 255, 0, 0 ]),
        Rgba([ 0, 255, 255, 64 ]),
        Rgba([ 255, 0, 255, 127 ]),
        Rgba([ 255, 255, 255, 255 ])
    ]);

    test_image("./_tests_output_/rec/a_albedo-ao.png", (1, 2), &[
        Rgba([ 255, 255, 255, 119 ]),
        Rgba([ 0, 0, 0, 185 ])
    ]);
}
