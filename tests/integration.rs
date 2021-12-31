use std::convert::TryInto;

fn read<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(path);
    std::fs::read(path)
}

#[test]
fn basic1() {
    let data = read("basic1.bin").unwrap();
    let (rest, result) = luabins::load(&data).unwrap();
    assert!(rest.is_empty());
    assert_eq!(
        result,
        vec![
            luabins::Value::Number(1f64.into()),
            luabins::Value::String("two".into()),
            luabins::Value::Table(vec![
                (
                    luabins::Key::Number(1f64.try_into().unwrap()),
                    luabins::Value::String("three".into())
                ),
                (
                    luabins::Key::Number(2f64.try_into().unwrap()),
                    luabins::Value::Number(4f64.into())
                )
            ])
        ]
    );
}

#[test]
fn round_trips() {
    let paths = (1..=5).map(|index| format!("basic{}.bin", index));
    for path in paths {
        let data = read(path).unwrap();
        let (rest, result) = luabins::load(&data).unwrap();
        assert!(rest.is_empty());
        let serialized = luabins::save(&result);
        assert_eq!(data, serialized);
    }
}
