import sharp from "sharp";

const svg = (await import("fs")).readFileSync("app-icon.svg");
await sharp(svg, { density: 220 })
  .resize(1024, 1024)
  .png()
  .toFile("app-icon.png");
console.log("wrote app-icon.png (1024)");
