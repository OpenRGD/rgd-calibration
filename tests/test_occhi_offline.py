"""Offline tests for non-actuating OCCHI helper; does not need camera or printer."""
import importlib.util
from pathlib import Path
import unittest

import cv2
import numpy as np

SPEC = importlib.util.spec_from_file_location(
    "occhi_stereo", Path(__file__).parents[1] / "tools/occhi_stereo/occhi_stereo.py")
OCCHI = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OCCHI)


class TestOcchiOffline(unittest.TestCase):
    def test_split_preserves_ab_order_when_rotated(self):
        frame = np.zeros((720, 2560, 3), np.uint8)
        frame[:360, :1280, 0] = 100
        frame[:360, 1280:, 0] = 200
        a, b = OCCHI.split_sbs(frame, 180)
        self.assertEqual(a.shape, (720, 1280, 3))
        self.assertEqual(int(a[-1, 0, 0]), 100)
        self.assertEqual(int(b[-1, 0, 0]), 200)
        self.assertEqual(int(a[0, 0, 0]), 0)
        self.assertEqual(int(b[0, 0, 0]), 0)

    def test_bad_sbs_shape_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "2560x720"):
            OCCHI.split_sbs(np.zeros((720, 1280, 3), np.uint8))

    def test_board_geometry(self):
        points = OCCHI.object_points((9, 6), 25)
        self.assertEqual(points.shape, (54, 3))
        self.assertAlmostEqual(float(points[-1, 0]), .2)
        self.assertAlmostEqual(float(points[-1, 1]), .125)

    def test_detect_generated_checkerboard(self):
        board = np.full((720, 1280, 3), 255, np.uint8)
        square = 60
        x0, y0 = 300, 140
        for row in range(7):
            for col in range(10):
                if (col + row) % 2 == 0:
                    cv2.rectangle(board,
                                  (x0+col*square, y0+row*square),
                                  (x0+(col+1)*square-1, y0+(row+1)*square-1),
                                  (0, 0, 0), -1)
        corners = OCCHI.detect_corners(board)
        self.assertIsNotNone(corners)
        self.assertEqual(corners.shape, (54, 1, 2))


if __name__ == '__main__':
    unittest.main()
