#!/usr/bin/env python3
"""CAL-0.3 experimental, read-only stereo capture and offline calibration.

Consumes one UVC side-by-side (SBS) frame. Never actuates or updates canonical RGD.
Image names A/B describe stream order AFTER individual image rotation, not
verified physical left/right orientation.
"""
from __future__ import annotations

import argparse
import json
import math
import platform
import sys
import time
from pathlib import Path

import cv2
import numpy as np

DEFAULT_DEVICE = "/dev/v4l/by-id/usb-3D_USB_Camera_3D_USB_Camera_01.00.00-video-index0"
PATTERN = (9, 6)  # inner corners (horizontal, vertical)


def split_sbs(frame: np.ndarray, rotate_each: int = 180) -> tuple[np.ndarray, np.ndarray]:
    """Split SBS into A/B; rotate halves separately, so stream order is preserved."""
    if frame is None or frame.ndim != 3 or frame.shape[2] != 3:
        raise ValueError("Expected a decoded BGR color image")
    height, width = frame.shape[:2]
    if width != 2560 or height != 720:
        raise ValueError(f"Expected SBS 2560x720; received {width}x{height}")
    if rotate_each not in (0, 180):
        raise ValueError("rotate_each must be 0 or 180")
    a, b = frame[:, :1280].copy(), frame[:, 1280:].copy()
    if rotate_each == 180:
        a = cv2.rotate(a, cv2.ROTATE_180)
        b = cv2.rotate(b, cv2.ROTATE_180)
    return a, b


def detect_corners(img: np.ndarray, pattern: tuple[int, int] = PATTERN):
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    if hasattr(cv2, "findChessboardCornersSB"):
        flags = cv2.CALIB_CB_NORMALIZE_IMAGE
        ok, corners = cv2.findChessboardCornersSB(gray, pattern, flags=flags)
        if ok:
            return corners.astype(np.float32)
    flags = cv2.CALIB_CB_ADAPTIVE_THRESH | cv2.CALIB_CB_NORMALIZE_IMAGE
    ok, corners = cv2.findChessboardCorners(gray, pattern, flags)
    if not ok:
        return None
    criteria = (cv2.TERM_CRITERIA_EPS | cv2.TERM_CRITERIA_MAX_ITER, 30, 0.001)
    return cv2.cornerSubPix(gray, corners, (11, 11), (-1, -1), criteria)


def object_points(pattern: tuple[int, int], square_mm: float):
    x, y = pattern
    pts = np.zeros((x * y, 3), np.float32)
    pts[:, :2] = np.mgrid[0:x, 0:y].T.reshape(-1, 2) * (square_mm / 1000.0)
    return pts


def _view_for_preview(a, b, corners_a, corners_b, pair_count):
    aa, bb = a.copy(), b.copy()
    for frame, points, label in ((aa, corners_a, "A"), (bb, corners_b, "B")):
        if points is not None:
            cv2.drawChessboardCorners(frame, PATTERN, points, True)
        status = "CHECKERBOARD OK" if points is not None else "NO BOARD"
        color = (0, 200, 0) if points is not None else (0, 0, 220)
        cv2.putText(frame, f"OCCHI {label} - {status}", (12, 34),
                    cv2.FONT_HERSHEY_SIMPLEX, .85, color, 2, cv2.LINE_AA)
    image = cv2.hconcat([aa, bb])
    image = cv2.resize(image, (1280, 360), interpolation=cv2.INTER_AREA)
    cv2.putText(image, f"c = capture; q = quit; saved={pair_count}",
                (12, 348), cv2.FONT_HERSHEY_SIMPLEX, .65, (0, 255, 255), 2)
    return image


def capture(args):
    session = Path(args.session).expanduser()
    session.mkdir(parents=True, exist_ok=True)
    device = args.device
    cap = cv2.VideoCapture(device, cv2.CAP_V4L2)
    if not cap.isOpened():
        raise RuntimeError(f"Unable to open camera {device}. Close ffplay or other clients.")
    cap.set(cv2.CAP_PROP_FOURCC, cv2.VideoWriter_fourcc(*"MJPG"))
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 2560)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 720)
    cap.set(cv2.CAP_PROP_FPS, 30)
    meta_file = session / "session.json"
    saved = len(list(session.glob("pair_*_A.png")))
    # Do not claim that user-space monotonic time is hardware exposure time.
    metadata = {
        "schema": "openrgd.experimental.stereo_capture.v0.1",
        "device_requested": device,
        "capture_resolution": [2560, 720],
        "requested_fps": 30,
        "inner_corners": list(PATTERN),
        "rotate_each_degrees": args.rotate_each,
        "sensor_order_verified": False,
        "timestamps": "host_monotonic_receipt_only_not_chronograf",
        "pairs": [],
        "non_actuating": True,
    }
    if meta_file.exists():
        metadata = json.loads(meta_file.read_text(encoding="utf-8"))
        if metadata.get("rotate_each_degrees") != args.rotate_each:
            raise RuntimeError("Existing session uses another rotation; choose a new session")
        saved = len(metadata.get("pairs", []))
    try:
        while True:
            ok, raw = cap.read()
            receipt_ns = time.monotonic_ns()
            if not ok:
                print("Camera frame unavailable", file=sys.stderr)
                continue
            a, b = split_sbs(raw, args.rotate_each)
            ca, cb = detect_corners(a), detect_corners(b)
            preview = _view_for_preview(a, b, ca, cb, saved)
            try:
                cv2.imshow("OCCHI - stereo checkerboard", preview)
                key = cv2.waitKey(1) & 0xFF
            except cv2.error as exc:
                raise RuntimeError("OpenCV GUI unavailable. Run from Ubuntu desktop session with display.") from exc
            if key in (ord('q'), 27):
                break
            if key == ord('c'):
                if ca is None or cb is None:
                    print("Not saved: 9x6 internal corners must be visible in BOTH eyes")
                    continue
                saved += 1
                stem = f"pair_{saved:03d}"
                path_a, path_b = session / f"{stem}_A.png", session / f"{stem}_B.png"
                if path_a.exists() or path_b.exists():
                    raise RuntimeError("Refusing to overwrite an existing pair")
                if not (cv2.imwrite(str(path_a), a) and cv2.imwrite(str(path_b), b)):
                    raise RuntimeError("Unable to save PNGs")
                metadata["pairs"].append({
                    "id": stem,
                    "a": path_a.name,
                    "b": path_b.name,
                    "host_receipt_monotonic_ns": receipt_ns,
                })
                meta_file.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
                print(f"Saved {stem}: {saved} valid pairs")
    finally:
        cap.release()
        cv2.destroyAllWindows()
    print(f"Session: {session} ({saved} pairs)")


def _load_pairs(session: Path):
    meta_path = session / "session.json"
    if not meta_path.is_file():
        raise RuntimeError(f"Missing {meta_path}; capture with this tool first")
    metadata = json.loads(meta_path.read_text(encoding="utf-8"))
    if metadata.get("schema") != "openrgd.experimental.stereo_capture.v0.1":
        raise RuntimeError("Unexpected session schema")
    for item in metadata.get("pairs", []):
        # Constrain metadata path to self-contained session files.
        aname, bname = item.get("a", ""), item.get("b", "")
        if Path(aname).name != aname or Path(bname).name != bname:
            raise RuntimeError("Pair references must be local file basenames")
        a = cv2.imread(str(session / aname), cv2.IMREAD_COLOR)
        b = cv2.imread(str(session / bname), cv2.IMREAD_COLOR)
        if a is None or b is None or a.shape != (720, 1280, 3) or b.shape != a.shape:
            raise RuntimeError(f"Corrupt or unexpected image pair: {aname}, {bname}")
        yield item["id"], a, b


def calibrate(args):
    session = Path(args.session).expanduser()
    out = Path(args.out).expanduser()
    if (out / "occhi_stereo_experimental.json").exists() and not args.overwrite:
        raise RuntimeError("Output calibration exists; use a new folder (or --overwrite)")
    if args.square_mm <= 0 or not math.isfinite(args.square_mm):
        raise ValueError("square-mm must be finite and >0")
    obj, image_a, image_b, used = [], [], [], []
    sample = None
    for name, a, b in _load_pairs(session):
        ca, cb = detect_corners(a), detect_corners(b)
        if ca is None or cb is None:
            print(f"SKIP {name}: checkerboard not found in both images")
            continue
        obj.append(object_points(PATTERN, args.square_mm))
        image_a.append(ca)
        image_b.append(cb)
        used.append(name)
        sample = (a, b)
    if len(obj) < args.min_pairs:
        raise RuntimeError(f"Only {len(obj)} valid pairs (need >= {args.min_pairs})."
                           " Move the board to different positions and inclinations.")
    size = (1280, 720)
    rms_a, k_a, d_a, *_ = cv2.calibrateCamera(obj, image_a, size, None, None)
    rms_b, k_b, d_b, *_ = cv2.calibrateCamera(obj, image_b, size, None, None)
    criteria = (cv2.TERM_CRITERIA_MAX_ITER | cv2.TERM_CRITERIA_EPS, 100, 1e-6)
    rms_stereo, k_a, d_a, k_b, d_b, r, t, e, f = cv2.stereoCalibrate(
        obj, image_a, image_b, k_a, d_a, k_b, d_b, size,
        criteria=criteria, flags=cv2.CALIB_FIX_INTRINSIC)
    r_a, r_b, p_a, p_b, q, roi_a, roi_b = cv2.stereoRectify(
        k_a, d_a, k_b, d_b, size, r, t,
        flags=cv2.CALIB_ZERO_DISPARITY, alpha=0)
    vertical = []
    for ca, cb in zip(image_a, image_b):
        pa = cv2.undistortPoints(ca, k_a, d_a, R=r_a, P=p_a).reshape(-1, 2)
        pb = cv2.undistortPoints(cb, k_b, d_b, R=r_b, P=p_b).reshape(-1, 2)
        vertical.extend(np.abs(pa[:, 1] - pb[:, 1]).tolist())
    baseline_mm_nominal = float(np.linalg.norm(t) * 1000.)
    med_y = float(np.median(vertical))
    p95_y = float(np.percentile(vertical, 95))
    verdict = "REVIEW_REQUIRED"
    if rms_stereo <= 1.5 and p95_y <= 2.5:
        verdict = "CANDIDATE_GOOD_GEOMETRY_NOT_PHYSICALLY_CERTIFIED"
    out.mkdir(parents=True, exist_ok=True)
    result = {
        "schema": "openrgd.experimental.stereo_estimate.v0.1",
        "status": verdict,
        "calibration_authority": "experimental_only_no_canonical_core_write",
        "physical_robot_instance": "Paky-01 - operator binding not attested",
        "camera_id": "occhi_32e4_2b10",
        "sensor_A_B_physical_side_verified": False,
        "sensor_A_B_stream_order_only": True,
        "input": {"resolution_sbs": [2560, 720], "per_eye": list(size),
                  "square_size_mm_declared": args.square_mm,
                  "square_size_physically_verified": bool(args.confirmed_square_mm),
                  "sessions": str(session), "pairs_used": used},
        "metrics": {"valid_pairs": len(used), "rms_A_px": float(rms_a),
                    "rms_B_px": float(rms_b), "rms_stereo_px": float(rms_stereo),
                    "rectified_vertical_median_px": med_y,
                    "rectified_vertical_p95_px": p95_y,
                    "baseline_mm_nominal_scale": baseline_mm_nominal},
        "camera_A": {"K": k_a.tolist(), "D": d_a.ravel().tolist(),
                     "R_rectify": r_a.tolist(), "P_rectify": p_a.tolist(),
                     "roi": list(roi_a)},
        "camera_B": {"K": k_b.tolist(), "D": d_b.ravel().tolist(),
                     "R_rectify": r_b.tolist(), "P_rectify": p_b.tolist(),
                     "roi": list(roi_b)},
        "stereo": {"R_A_to_B": r.tolist(), "T_A_to_B_m_nominal_scale": t.ravel().tolist(),
                   "Q": q.tolist()},
        "notes": [
            "Uncertified candidate. RMS is fitting error, not proof of physical accuracy.",
            "Depth metric scale depends on verified physical checkerboard square size.",
            "Image capture timestamps are host receipt time, not synchronized sensor exposure times.",
            "Do not send this directly as a validated CAL profile or physical safety authority.",
        ],
        "software": {"opencv": cv2.__version__, "python": platform.python_version()},
    }
    outfile = out / "occhi_stereo_experimental.json"
    outfile.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    if sample is not None:
        ra, rb = rectify_pair(sample[0], sample[1], result)
        joined = cv2.hconcat([ra, rb])
        for y in range(40, joined.shape[0], 60):
            cv2.line(joined, (0, y), (joined.shape[1]-1, y), (0, 255, 0), 1)
        cv2.imwrite(str(out / "rectified_example.jpg"), joined)
    print(json.dumps({"file": str(outfile), "metrics": result["metrics"],
                      "status": verdict}, indent=2))


def rectify_pair(a, b, calibration):
    size = (1280, 720)
    result = []
    for eye, img in (("camera_A", a), ("camera_B", b)):
        c = calibration[eye]
        xmap, ymap = cv2.initUndistortRectifyMap(
            np.asarray(c["K"], dtype=np.float64),
            np.asarray(c["D"], dtype=np.float64),
            np.asarray(c["R_rectify"], dtype=np.float64),
            np.asarray(c["P_rectify"], dtype=np.float64),
            size, cv2.CV_32FC1)
        result.append(cv2.remap(img, xmap, ymap, cv2.INTER_LINEAR))
    return result[0], result[1]


def rectify(args):
    info = json.loads(Path(args.calibration).expanduser().read_text(encoding="utf-8"))
    if info.get("schema") != "openrgd.experimental.stereo_estimate.v0.1":
        raise RuntimeError("Unexpected calibration schema")
    frame = cv2.imread(str(Path(args.frame).expanduser()), cv2.IMREAD_COLOR)
    rotate = args.rotate_each
    a, b = split_sbs(frame, rotate)
    aa, bb = rectify_pair(a, b, info)
    joined = cv2.hconcat([aa, bb])
    for y in range(40, joined.shape[0], 60):
        cv2.line(joined, (0, y), (joined.shape[1]-1, y), (0, 255, 0), 1)
    dst = Path(args.out).expanduser()
    if dst.exists() and not args.overwrite:
        raise RuntimeError(f"Refusing to overwrite {dst}")
    dst.parent.mkdir(parents=True, exist_ok=True)
    if not cv2.imwrite(str(dst), joined):
        raise RuntimeError(f"Unable to write {dst}")
    print(f"Wrote {dst}")


def main(argv=None):
    parser = argparse.ArgumentParser(description="OCCHI stereo guided camera calibration - passive only")
    sub = parser.add_subparsers(dest="command", required=True)
    c = sub.add_parser("capture", help="Preview, detect checkerboard, and save paired frames using c key")
    c.add_argument("--device", default=DEFAULT_DEVICE)
    c.add_argument("--session", default="~/occhi-calibration/session01")
    c.add_argument("--rotate-each", type=int, choices=(0, 180), default=180)
    c.set_defaults(func=capture)
    cal = sub.add_parser("calibrate", help="Offline estimate intrinsics, stereo extrinsics, and rectification")
    cal.add_argument("--session", default="~/occhi-calibration/session01")
    cal.add_argument("--out", default="~/occhi-calibration/estimate01")
    cal.add_argument("--square-mm", type=float, default=25.0)
    cal.add_argument("--confirmed-square-mm", action="store_true",
                     help="Only set after independently measuring printed square edge with a ruler")
    cal.add_argument("--min-pairs", type=int, default=18)
    cal.add_argument("--overwrite", action="store_true")
    cal.set_defaults(func=calibrate)
    r = sub.add_parser("rectify", help="Rectify one captured raw SBS image after calibration")
    r.add_argument("--calibration", required=True)
    r.add_argument("--frame", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--rotate-each", type=int, choices=(0, 180), default=180)
    r.add_argument("--overwrite", action="store_true")
    r.set_defaults(func=rectify)
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except (ValueError, RuntimeError, cv2.error, OSError) as exc:
        parser.exit(2, f"ERROR: {exc}\n")


if __name__ == "__main__":
    main()
