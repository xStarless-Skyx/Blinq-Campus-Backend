const fs = require("fs");
const path = require("path");
const express = require("express");
const tf = require("@tensorflow/tfjs-node");
const nsfwjs = require("nsfwjs");
const tesseract = require("tesseract.js");

const PORT = Number(process.env.PORT || 14820);
const NSFW_THRESHOLD = Number(process.env.BLINQ_IMAGE_NSFW_THRESHOLD || 0.6);
const OCR_ENABLED = (process.env.BLINQ_IMAGE_OCR_ENABLED || "true") === "true";
const OCR_MAX_PIXELS = Number(process.env.BLINQ_IMAGE_OCR_MAX_PIXELS || 4000000);

const WORDS_PATH = process.env.BLINQ_BAD_WORDS_PATH
    ? path.resolve(process.env.BLINQ_BAD_WORDS_PATH)
    : path.resolve(__dirname, "../../crates/delta/resources/words.json");

function normalizeText(text) {
    let out = "";
    let lastSpace = false;
    for (const ch of text) {
        if (/[A-Za-z0-9]/.test(ch)) {
            out += ch.toLowerCase();
            lastSpace = false;
        } else if (!lastSpace) {
            out += " ";
            lastSpace = true;
        }
    }
    return out.trim();
}

function loadWordList() {
    try {
        const raw = fs.readFileSync(WORDS_PATH, "utf8");
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed)) return [];
        return parsed
            .map((entry) => String(entry).trim())
            .filter((entry) => entry.length > 0);
    } catch (err) {
        console.warn("Failed to load word list:", err.message);
        return [];
    }
}

const WORDS = loadWordList();
const SINGLE_TERMS = new Set();
const PHRASES = new Set();
for (const raw of WORDS) {
    const normalized = normalizeText(raw);
    if (!normalized) continue;
    if (normalized.includes(" ")) {
        PHRASES.add(normalized);
    } else {
        SINGLE_TERMS.add(normalized);
    }
}

function findProfanityMatches(text) {
    const normalized = normalizeText(text);
    if (!normalized) return [];

    const matches = new Set();
    for (const token of normalized.split(/\s+/)) {
        if (SINGLE_TERMS.has(token)) matches.add(token);
    }

    if (PHRASES.size > 0) {
        const padded = ` ${normalized} `;
        for (const phrase of PHRASES) {
            const needle = ` ${phrase} `;
            if (padded.includes(needle)) matches.add(phrase);
        }
    }

    return Array.from(matches);
}

let modelPromise = null;
let ocrWorkerPromise = null;

async function getModel() {
    if (!modelPromise) {
        modelPromise = nsfwjs.load();
    }
    return modelPromise;
}

async function getOcrWorker() {
    if (!OCR_ENABLED) return null;
    if (!ocrWorkerPromise) {
        ocrWorkerPromise = (async () => {
            const worker = await tesseract.createWorker("eng");
            return worker;
        })();
    }
    return ocrWorkerPromise;
}

const app = express();
app.use(
    "/scan",
    express.raw({ type: "*/*", limit: "20mb" })
);

app.post("/scan", async (req, res) => {
    try {
        if (!req.body || req.body.length === 0) {
            return res.status(400).json({ error: "empty body" });
        }

        const buffer = req.body;
        const tensor = tf.node.decodeImage(buffer, 3);

        const [height, width] = tensor.shape;
        const model = await getModel();
        const predictions = await model.classify(tensor);
        tf.dispose(tensor);

        let nsfwLabel = null;
        let nsfwScore = null;
        let nsfw = null;

        if (predictions && predictions.length > 0) {
            const top = predictions.reduce((best, entry) =>
                entry.probability > best.probability ? entry : best
            );
            nsfwLabel = top.className;
            nsfwScore = top.probability;
            if (["Porn", "Hentai", "Sexy"].includes(top.className)) {
                nsfw = top.probability >= NSFW_THRESHOLD;
            } else {
                nsfw = false;
            }
        }

        let profanity = null;
        let profanityMatches = null;

        if (OCR_ENABLED && width * height <= OCR_MAX_PIXELS) {
            const worker = await getOcrWorker();
            if (worker) {
                const result = await worker.recognize(buffer);
                const text = result?.data?.text || "";
                const matches = findProfanityMatches(text);
                profanityMatches = matches.length > 0 ? matches : [];
                profanity = matches.length > 0;
            }
        }

        return res.json({
            nsfw,
            nsfw_label: nsfwLabel,
            nsfw_score: nsfwScore,
            profanity,
            profanity_matches: profanityMatches,
        });
    } catch (err) {
        console.error("Moderation error:", err);
        return res.status(500).json({ error: "moderation failed" });
    }
});

app.listen(PORT, () => {
    console.log(`Image moderation service listening on ${PORT}`);
});
