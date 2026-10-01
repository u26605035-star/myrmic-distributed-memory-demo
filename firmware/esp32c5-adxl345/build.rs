fn main() {
    esp_firmware_build::configure();

    esp_firmware_build::pipeline()
        .board("board.yml")
        .pipeline("pipeline.yml")
        .include("../../signal-modules")
        .generate();
}
