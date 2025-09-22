use cranelift_codegen::ir::Function;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let file = args.get(1).expect("Usage: pvmc dump <file>");
    let bytes = std::fs::read(file).expect("Failed to read file");
    let (clif, _) =
        postcard::from_bytes::<(Function, bool)>(&bytes).expect("Failed to parse module");
    println!("{}", clif.display());
}
