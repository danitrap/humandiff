fn main() {
    // execute os command git diff > diff.txt
    let output = std::process::Command::new("git")
        .arg("diff")
        .output()
        .expect("failed to execute process");
    // write output to file
    std::fs::write("diff.txt", output.stdout).expect("Unable to write file");

}
