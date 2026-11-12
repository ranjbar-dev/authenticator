# Bundled fonts

Unmodified Noto Sans Regular (400) and SemiBold (600), distributed under the
SIL Open Font License 1.1 in [OFL.txt](OFL.txt). These weights are selected by
the shared Rust typography roles; egui `RichText::strong` changes color and is
not used to simulate font weight.

Fetched 12 November 2026 from the official Noto repository:

- [NotoSans-Regular.ttf](https://github.com/notofonts/noto-fonts/blob/main/hinted/ttf/NotoSans/NotoSans-Regular.ttf)
- [NotoSans-SemiBold.ttf](https://github.com/notofonts/noto-fonts/blob/main/hinted/ttf/NotoSans/NotoSans-SemiBold.ttf)
- [License](https://github.com/notofonts/noto-fonts/blob/main/LICENSE)

The font files are embedded at build time. No runtime download or system-font
lookup occurs. Packaging must retain this license with other notices.
