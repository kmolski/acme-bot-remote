const fs = require("fs");
const esbuild = require("esbuild");
const pkg = require("./package.json");

fs.mkdirSync("./dist_js", { recursive: true });

esbuild.build({
    entryPoints: Object.keys(pkg.dependencies),
    bundle: true,
    minify: true,
    outfile: "dist_js/stomp.js",
    format: "esm",
}).catch(() => process.exit(1));
