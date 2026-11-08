# Office's default theme maps the Gujarati script to a typeface whose name is also a person's name.
# The same word hashes as presentation/build_deck.py, so no name is written here in clear.
NAME_HASHES = {
    "583a663b7f10962c226e8e626edda153ade1714c3f47875e18ca1cbdcdb648e7", "cfbff47427fb68609fc3d9b0c28b81a6b4d7621a5c81cf5170fb7da07b92c54f",
    "44ad9af1e203cc5b9c3d9433063d1b3daee23af0dd68fa7c1aa1567b20272961", "976e761dab368049ebdd82e3dc0e390d19d452db0f708d11314d31739c9dffd0",
    "147697aaab1c836b65edab8d89099758ea001c7b0d3c7a8bad38af6256aaa524", "d54170f31396d8c2541e1d527b83839b03255205e6a3b190e3d895d105ad2c42",
    "0e74acfc91692058bfe94761e22729b1bf2021ef8548e525107222869a6c7448", "6d12f211cccc24fb97ae3e07f19bee9332aef9fa3dafe9cbbb7f3bbc7b60d3ce",
}
FONT_ENTRY = re.compile(rb'<a:font script="[^"]*" typeface="([^"]*)"/>')


def scrub_fonts(xml):
    """Drop each theme font entry whose typeface name is a hashed name word, as build_deck.py scrub_fonts does."""
    def keep(m):
        words = re.findall(r"[A-Za-z]+", m.group(1).decode("utf-8", "replace"))
        return b"" if any(hashlib.sha256(w.lower().encode()).hexdigest() in NAME_HASHES for w in words) else m.group(0)
    return FONT_ENTRY.sub(keep, xml)


def finalize_docx(path):
    """After the last Word save, in one re-zip: author and last modifier back to the author (Word stamps
    its own user on save), no company or manager in the app properties, and the theme font scrub.
    Returns the number of theme font entries removed."""
    tmp = path.with_suffix(".tmp.docx")
    removed = 0
    with zipfile.ZipFile(path) as zin, zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            data = zin.read(item.filename)
            if item.filename == "docProps/core.xml":
                text = data.decode("utf-8")
                text = re.sub(r"<cp:lastModifiedBy>.*?</cp:lastModifiedBy>", f"<cp:lastModifiedBy>{AUTHOR}</cp:lastModifiedBy>", text)
                text = re.sub(r"<dc:creator>.*?</dc:creator>", f"<dc:creator>{AUTHOR}</dc:creator>", text)
                data = text.encode("utf-8")
            elif item.filename == "docProps/app.xml":
                text = data.decode("utf-8")
                for tag in ("Company", "Manager"):
                    text = re.sub(rf"<{tag}>.*?</{tag}>", f"<{tag}></{tag}>", text)
                data = text.encode("utf-8")
            elif "/theme/" in item.filename and item.filename.endswith(".xml"):
                new = scrub_fonts(data)
                removed += len(FONT_ENTRY.findall(data)) - len(FONT_ENTRY.findall(new))
                data = new
            zout.writestr(item, data)
    tmp.replace(path)
    return removed


def word(args):
    return run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(B / "word_run.ps1"), *args]).returncode


def word_step():
    """Word saves the docx with the TOC and fields updated; the docx is then finalised; Word opens the
    final docx read-only and exports the PDF from it, so the PDF and the docx are the same file."""
    lock_env = os.environ.get("NUSH2_WORD_LOCK")
    if not lock_env:
        print("NUSH2_WORD_LOCK is not set; the Word step needs the shared lock and did not run")
        return 4
    lock = Path(lock_env)
    if not take_lock(lock):
        return 5
    try:
        code = word(["-InDocx", str(RAW), "-OutDocx", str(DOCX), "-OutPdf", str(PDF), "-TimeoutSec", "300"])
        if code != 0:
            print(f"Word save step failed with code {code}")
            return code
        removed = finalize_docx(DOCX)
        print(f"docx finalised after the last Word save: {removed} theme font entries removed")
        code = word(["-InDocx", str(DOCX), "-OutDocx", str(DOCX), "-OutPdf", str(PDF), "-TimeoutSec", "300", "-ExportOnly"])
        if code != 0:
            print(f"Word export step failed with code {code}")
            return code
    finally:
        shutil.rmtree(lock, ignore_errors=True)
    (B / "logs" / "pdf_route.txt").write_text("P1, ExportAsFixedFormat with heading bookmarks, from the final docx opened read-only",
                                              encoding="utf-8")
    return 0
