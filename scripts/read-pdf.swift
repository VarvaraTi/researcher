import Foundation
import PDFKit

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data(("read-pdf: " + message + "\n").utf8))
    exit(1)
}

let usage = "Usage: sh scripts/read-pdf.sh FILE.pdf [--pages START:END | --all | --json]\nDefault: first 12 PDF pages. Page numbers are 1-based, not printed book numbers."
var args = Array(CommandLine.arguments.dropFirst())
// Some older Swift interpreters expose the frontend's entire argument list.
if URL(fileURLWithPath: CommandLine.arguments[0]).lastPathComponent == "swift-frontend",
   let separator = args.firstIndex(of: "--") {
    args = Array(args.dropFirst(separator + 1))
}
while args.first == "--" { args.removeFirst() }
if args == ["--help"] { print(usage); exit(0) }
guard let path = args.first, [1, 2, 3].contains(args.count) else { fail(usage) }
if args.count == 2 && args[1] == "--json" {
    let url = URL(fileURLWithPath: path).standardizedFileURL
    var result: [String: Any] = ["extractor": "PDFKit", "extractor_version": "read-pdf-json-v1", "page_count": NSNull(), "pages": []]
    if !FileManager.default.isReadableFile(atPath: url.path) {
        result["document_failure"] = "File is not readable."
    } else if let document = PDFDocument(url: url) {
        if document.isLocked {
            result["document_failure"] = "PDF is password locked."
        } else if document.pageCount < 1 {
            result["document_failure"] = "PDF has no discoverable pages."
        } else {
            result["page_count"] = document.pageCount
            var pages: [[String: Any]] = []
            for number in 1...document.pageCount {
                guard let page = document.page(at: number - 1) else {
                    pages.append(["page": number, "state": "failed", "reason": "PDFKit could not read page."])
                    continue
                }
                let text = page.string ?? ""
                if text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    pages.append(["page": number, "state": "no_text"])
                } else {
                    pages.append(["page": number, "state": "text", "text": text])
                }
            }
            result["pages"] = pages
        }
    } else {
        result["document_failure"] = "PDFKit could not open the PDF."
    }
    do {
        let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data("\n".utf8))
        exit(0)
    } catch { fail("Cannot encode extraction result: \(error)") }
}
var start = 1
var requestedEnd: Int? = nil
var all = false
if args.count == 2 {
    guard args[1] == "--all" else { fail(usage) }
    all = true
} else if args.count == 3 {
    let parts = args[2].split(separator: ":", omittingEmptySubsequences: false)
    guard args[1] == "--pages", parts.count == 2,
          let first = Int(parts[0]), let last = Int(parts[1]),
          first > 0, last >= first else { fail("Invalid page range.\n" + usage) }
    start = first
    requestedEnd = last
}
let url = URL(fileURLWithPath: path).standardizedFileURL
guard FileManager.default.isReadableFile(atPath: url.path) else { fail("File is not readable: \(url.path)") }
guard let document = PDFDocument(url: url) else { fail("Cannot open PDF: \(url.path)") }
guard !document.isLocked else { fail("PDF is password locked.") }
let end = requestedEnd ?? (all ? document.pageCount : min(12, document.pageCount))
guard start <= end, end <= document.pageCount else { fail("Range exceeds document (\(document.pageCount) pages).") }
print("SOURCE: \(url.path)\nPDF PAGES: \(document.pageCount)\nEXTRACTED RANGE: \(start):\(end)\nMODE: text only; no OCR or image inspection")
for number in start...end {
    guard let page = document.page(at: number - 1) else { fail("Cannot read page \(number). Output is incomplete.") }
    let text = page.string ?? ""
    print("\n--- PDF PAGE \(number) ---")
    if text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
        print("[No extractable text; page may be blank or image-only.]")
    } else {
        print(text)
    }
}
