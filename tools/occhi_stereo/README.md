# CAL-0.3 EXPERIMENTAL — OCCHI stereo (Paky-01)

Non-actuating passive capture and OpenCV stereo-calibration experiment for the 3D USB Camera (VID:PID 32e4:2b10). This is NOT an approved CAL-0.3 release nor a physically certified calibration profile. Outputs must be reviewed before lifecycle integration.

## Setup (Ubuntu desktop)

```bash
cd ~/openrgd/repos/rgd-calibration
python3 -m venv .venv-occhi
source .venv-occhi/bin/activate
python -m pip install -r tools/occhi_stereo/requirements.txt
```

Print the A4 landscape 10x7-square (9x6 internal corners) checkerboard, 25 mm square, at **actual size 100%**. The ready-to-print PDF is supplied separately in the companion download. To regenerate it:

```bash
python -m pip install reportlab
python tools/occhi_stereo/make_board.py --out ~/board-A4-25mm.pdf
```

Measure actual printed square size. Affix the paper to a stiff, flat panel.

## Capture

**Manual camera capture is an isolated experiment, not a shared producer.** Confirm the host/Peripheral/Reality Ingress acquisition owner has released this camera. Do not run a second UVC reader or invoke `capture` from RI. The acknowledgment flag is only an operator declaration, **not a technical lease**.

```bash
python tools/occhi_stereo/occhi_stereo.py capture \
  --session ~/occhi-calibration/session01 \
  --square-mm 25 \
  --confirm-exclusive-camera-owner
```

For the independent 20 mm board used in other experiments, use `--square-mm 20` and a **separate session**. Do not copy 20 mm observations into 25 mm datasets. `--confirmed-square-mm` is an operator claim after physical measurement, not a certificate.

Press `c` only when BOTH eyes detect the board; press `q` when done. Collect 25–35 diverse board poses and distances; avoid motion blur. Preserve fixed physical geometry within the stereo module. The default rotation corrects the camera's currently inverted mounting by rotating **each** eye individually, retaining stream order A/B.

## Estimate & review

```bash
python tools/occhi_stereo/occhi_stereo.py calibrate \
  --session ~/occhi-calibration/session01 \
  --out ~/occhi-calibration/estimate01 \
  --square-mm 25
```

Use the **same declared square size recorded in the capture session**, otherwise the estimator rejects it. Legacy sessions missing board-size metadata require a separate reviewed migration, not a guess. Only add `--confirmed-square-mm` after independent physical measurement. The result always keeps `square_size_physically_verified=false`: a CLI checkbox is not independent metrological attestation. Outputs are `occhi_stereo_experimental.json` and `rectified_example.jpg`; existing estimates are not overwritten. Review stereo RMS, vertical p95 residual, visual guide-line alignment and plausible physical baseline. Re-acquire more views if inconsistent.

To rectify a known 2560x720 raw side-by-side image:

```bash
python tools/occhi_stereo/occhi_stereo.py rectify \
  --calibration ~/occhi-calibration/estimate01/occhi_stereo_experimental.json \
  --frame ~/occhi-test.jpg \
  --out ~/occhi-test-rectified.jpg
```

## Architectural boundary

- No commands to motors or CAN, no Canonical Core writes, no automatic state promotion.
- Operator-acknowledged camera ownership does not replace a host broker lease; this tool is unsuitable for parallel/shared RI acquisition. It never runs on importing a model or loading CAL.
- CI is synthetic; real host capture and independent metrological validation remain pending.
- Camera A/B denotes image-stream order; physical left/right orientation still requires verification.
- Stored timestamps are host monotonic receipt timestamps, not hardware exposure or Chronograf synchronization.
- Keep image datasets local (faces, laboratory), and do not commit captures to Git.
- Recheck stereo if module hardware changes. Future neck mount needs separate camera-to-body extrinsic calibration.
- This workflow belongs to the CAL-0.3 guided measurements roadmap and remains experimental pending formal profile adaptation and validation.
