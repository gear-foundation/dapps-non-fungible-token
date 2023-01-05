fn main() {
    gear_wasm_builder::build();
    gear_wasm_builder::build_with_metadata::<nft_io::NFTMetadata>()
}
