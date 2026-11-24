"""Regenerate the document fixtures with real-world producers.

Run from the repo root:  python3 tests/fixtures/docs/make_fixtures.py
Needs: python-docx, openpyxl, python-pptx, reportlab, LibreOffice (soffice),
pandoc, and a Chromium binary (CHROMIUM env; for a CID-font PDF like modern apps emit).
"""
import datetime, os, shutil, subprocess, tempfile

OUT = os.path.dirname(os.path.abspath(__file__))

def contract_docx():
    from docx import Document
    d = Document()
    d.add_heading("Master Services Agreement", 0)
    d.add_paragraph("This Agreement is made between Northwind Traders (\"Customer\") and Contoso Ltd (\"Provider\").")
    d.add_heading("1. Term and Renewal", level=1)
    d.add_paragraph("The initial term is twelve (12) months starting 2026-01-01 and renews automatically for successive one-year terms unless either party gives 60 days written notice.")
    d.add_heading("2. Fees", level=1)
    d.add_paragraph("Monthly fee of $4,500 payable net 30.", style="List Number")
    d.add_paragraph("Late payments accrue 1.5% interest per month.", style="List Number")
    d.add_heading("3. Liability", level=1)
    d.add_paragraph("Provider's total liability is limited to fees paid in the prior 12 months.")
    d.add_paragraph("Excludes gross negligence", style="List Bullet")
    d.add_paragraph("Excludes confidentiality breaches", style="List Bullet")
    t = d.add_table(rows=3, cols=3)
    for i, row in enumerate([["Milestone", "Date", "Amount"], ["Kickoff", "2026-01-15", "$9,000"], ["Go-live", "2026-04-01", "$13,500"]]):
        for j, v in enumerate(row):
            t.cell(i, j).text = v
    d.save(os.path.join(OUT, "contract.docx"))

def memo_docx():
    md = "# Q3 Planning Memo\n\n## Decisions\n\n- Hire two support engineers\n- Move billing to the new vendor\n\n## Open questions\n\n1. Who owns the migration?\n2. What is the budget cap?\n\nSee **section 4** of the *operating plan*.\n"
    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, "memo.md")
        open(src, "w").write(md)
        subprocess.run(["pandoc", src, "-o", os.path.join(OUT, "memo-pandoc.docx")], check=True)

def expenses_xlsx():
    import openpyxl
    from openpyxl.styles import Font
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = "Expenses"
    ws.append(["Date", "Vendor", "Category", "Amount", "Receipt"])
    rows = [
        (datetime.date(2026, 10, 3), "Acme Office Supply", "Office", 182.40, True),
        (datetime.date(2026, 10, 9), "Skyline Travel", "Travel", 1250.00, True),
        (datetime.date(2026, 10, 9), "Skyline Travel", "Travel", 1250.00, False),
        (datetime.date(2026, 10, 17), "Blue Bottle", "Meals", 36.75, True),
    ]
    for r in rows:
        ws.append(list(r))
    for c in ws["A"][1:]:
        c.number_format = "yyyy-mm-dd"
    ws["A1"].font = Font(bold=True)
    ws.append([None, "Total", None, "=SUM(D2:D5)", None])
    notes = wb.create_sheet("Notes")
    notes["A1"] = "Duplicate Skyline charge under dispute"
    notes["A2"] = "Policy: receipts required over $25"
    with tempfile.TemporaryDirectory() as td:
        raw = os.path.join(td, "expenses.xlsx")
        wb.save(raw)
        # Re-save through LibreOffice so formula cells carry cached values,
        # exactly like a workbook saved by Excel.
        subprocess.run(["soffice", "--headless", "--calc", "--convert-to", "xlsx:Calc MS Excel 2007 XML", "--outdir", os.path.join(td, "lo"), raw], check=True, capture_output=True)
        shutil.copy(os.path.join(td, "lo", "expenses.xlsx"), os.path.join(OUT, "expenses.xlsx"))

def deck_pptx():
    from pptx import Presentation
    p = Presentation()
    s1 = p.slides.add_slide(p.slide_layouts[0])
    s1.shapes.title.text = "Q4 Launch Plan"
    s1.placeholders[1].text = "Ops review, November 2026"
    s1.notes_slide.notes_text_frame.text = "Open with the revenue chart."
    s2 = p.slides.add_slide(p.slide_layouts[1])
    s2.shapes.title.text = "Milestones"
    body = s2.placeholders[1].text_frame
    body.text = "Beta on Dec 1"
    body.add_paragraph().text = "GA on Jan 15"
    p.save(os.path.join(OUT, "deck.pptx"))

def invoice_pdf():
    from reportlab.lib.pagesizes import letter
    from reportlab.pdfgen import canvas
    c = canvas.Canvas(os.path.join(OUT, "invoice-reportlab.pdf"), pagesize=letter)
    c.setFont("Helvetica-Bold", 16)
    c.drawString(72, 720, "INVOICE INV-2026-0042")
    c.setFont("Helvetica", 11)
    c.drawString(72, 690, "Bill to: Northwind Traders")
    c.drawString(72, 670, "Consulting services, October 2026: $4,500.00")
    c.drawString(72, 650, "Due date: 2026-11-30")
    c.showPage()
    c.setFont("Helvetica", 11)
    c.drawString(72, 720, "Payment terms: net 30. Late fee 1.5% per month.")
    c.save()

def report_pdf_chromium():
    html = """<html><head><meta charset='utf-8'></head><body style='font-family: sans-serif'><h1>Weekly Ops Report</h1>
    <p>Tickets closed: 128 (up 12%). Café expansion approved — naïve estimates revised.</p>
    <table border=1><tr><th>Team</th><th>Open</th></tr><tr><td>Billing</td><td>7</td></tr><tr><td>Support</td><td>19</td></tr></table>
    </body></html>"""
    chrome = os.environ.get("CHROMIUM", "/opt/pw-browsers/chromium")
    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, "report.html")
        open(src, "w", encoding="utf-8").write(html)
        subprocess.run([chrome, "--headless", "--no-sandbox", "--disable-gpu", "--no-pdf-header-footer",
                        "--print-to-pdf=" + os.path.join(OUT, "report-chromium.pdf"), "file://" + src],
                       check=True, capture_output=True)

if __name__ == "__main__":
    contract_docx(); memo_docx(); expenses_xlsx(); deck_pptx(); invoice_pdf(); report_pdf_chromium()
    print("fixtures written to", OUT)
