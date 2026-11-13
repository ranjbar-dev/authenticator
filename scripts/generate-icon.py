"""Regenerate the original vector-derived Windows icon (development only)."""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SCALE = 4
image = Image.new("RGBA", (256 * SCALE, 256 * SCALE))
draw = ImageDraw.Draw(image)
draw.rounded_rectangle((32, 32, 992, 992), radius=208, fill="#4b8df8")
draw.ellipse((50*SCALE, 52*SCALE, 138*SCALE, 140*SCALE), outline="white", width=20*SCALE)
for a, b in [((119,121),(190,192)),((156,158),(176,138)),((176,178),(196,158))]:
    draw.line((a[0]*SCALE,a[1]*SCALE,b[0]*SCALE,b[1]*SCALE), fill="white", width=20*SCALE)
    for x,y in (a,b):
        draw.ellipse(((x-10)*SCALE,(y-10)*SCALE,(x+10)*SCALE,(y+10)*SCALE),fill="white")
image = image.resize((256,256), Image.Resampling.LANCZOS)
image.save(ROOT / "assets/authenticator.png")
image.save(ROOT / "assets/authenticator.ico", sizes=[(n,n) for n in (16,24,32,48,64,128,256)])
