fn main() {
    embed_resource::compile("resources/icon.rc", embed_resource::NONE).manifest_required().unwrap();
}
