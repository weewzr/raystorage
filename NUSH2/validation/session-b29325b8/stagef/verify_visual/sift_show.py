import sys, zipfile
import numpy as np, cv2
z = zipfile.ZipFile(sys.argv[1]); out = sys.argv[4]
q = cv2.imdecode(np.frombuffer(z.read("ppt/media/" + sys.argv[2]), np.uint8), cv2.IMREAD_COLOR)
p = cv2.imread(sys.argv[3], cv2.IMREAD_COLOR); s = 2048 / p.shape[1]; p = cv2.resize(p, (2048, int(p.shape[0] * s)))
sift = cv2.SIFT_create(nfeatures=6000)
kq, dq = sift.detectAndCompute(cv2.cvtColor(q, cv2.COLOR_BGR2GRAY), None); kp, dp = sift.detectAndCompute(cv2.cvtColor(p, cv2.COLOR_BGR2GRAY), None)
m = cv2.BFMatcher().knnMatch(dq, dp, k=2); good = [a for a, b in (x for x in m if len(x) == 2) if a.distance < 0.75 * b.distance]
src = np.float32([kq[g.queryIdx].pt for g in good]); dst = np.float32([kp[g.trainIdx].pt for g in good])
H, mask = cv2.findHomography(src, dst, cv2.RANSAC, 6.0)
h, w = q.shape[:2]
box = cv2.perspectiveTransform(np.float32([[0, 0], [w, 0], [w, h], [0, h]]).reshape(-1, 1, 2), H)
cv2.polylines(p, [np.int32(box)], True, (0, 0, 255), 6)
warp = cv2.warpPerspective(p, np.linalg.inv(H), (w, h))
qq = cv2.resize(q, (int(w * 600 / h), 600)); ww = cv2.resize(warp, (int(w * 600 / h), 600)); pp = cv2.resize(p, (int(p.shape[1] * 600 / p.shape[0]), 600))
cv2.imwrite(out, np.hstack([pp, qq, ww])); print("inliers", int(mask.sum()), "box", np.int32(box).reshape(-1, 2).tolist())
