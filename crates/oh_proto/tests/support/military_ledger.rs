pub fn maximum_pack(root: std::path::PathBuf) -> (std::path::PathBuf, String) {
    let path = root.join("common/military/synthetic.toml");
    let combat = "c".repeat(64);
    let support = "s".repeat(64);
    let template = "t".repeat(64);
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("example", &template)
        .replace("\"line\"", &format!("\"{combat}\""))
        .replace("\"support\"", &format!("\"{support}\""))
        // Restore the role token, which is not a component identifier.
        .replace(&format!("role = \"{support}\""), "role = \"support\"");
    let source = source
        .replace(
            &format!("combat = [\"{combat}\"]"),
            &format!("combat = [{}]", vec![format!("\"{combat}\""); 12].join(",")),
        )
        .replace(
            &format!("support = [\"{support}\"]"),
            &format!(
                "support = [{}]",
                vec![format!("\"{support}\""); 4].join(",")
            ),
        );
    let fields = [
        "strength",
        "soft_fire",
        "hard_fire",
        "defense",
        "breakthrough",
        "frontage",
        "supply_use",
    ];
    let source = source
        .lines()
        .map(|line| {
            if fields
                .iter()
                .any(|field| line.starts_with(&format!("{field} = ")))
            {
                format!("{} = \"8000000000000\"", line.split(" = ").next().unwrap())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, source).unwrap();
    (root, template)
}
