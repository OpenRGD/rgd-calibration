#!/usr/bin/env python3
"""True-size 10x7 squares at 25 mm for A4 landscape. No resizing on print."""
import argparse
from reportlab.lib.pagesizes import A4, landscape
from reportlab.lib.units import mm
from reportlab.pdfgen import canvas


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', default='board-A4-25mm.pdf')
    args = parser.parse_args()
    width, height = landscape(A4)
    square = 25 * mm
    cols, rows = 10, 7
    x0, y0 = (width-cols*square)/2, (height-rows*square)/2
    c = canvas.Canvas(args.out, pagesize=(width, height), pageCompression=0)
    c.setTitle('OCCHI - A4 stereo calibration board 10x7 squares, 25mm')
    for row in range(rows):
        for col in range(cols):
            c.setFillColorRGB(0, 0, 0) if (col + row) % 2 == 0 else c.setFillColorRGB(1, 1, 1)
            c.rect(x0 + col*square, y0 + (rows-1-row)*square,
                   square, square, fill=1, stroke=0)
    c.showPage()
    c.save()
    print(args.out)


if __name__ == '__main__':
    main()
