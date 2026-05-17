import cv2
import numpy as np
from PIL import Image
import io
import base64


class VisionProcessor:
    def __init__(self, quality: int = 95, max_size: int = 4096):
        self.quality = quality
        self.max_size = max_size

    def _ensure_max_size(self, image: Image.Image) -> Image.Image:
        w, h = image.size
        if w <= self.max_size and h <= self.max_size:
            return image
        scale = self.max_size / max(w, h)
        new_w = int(w * scale)
        new_h = int(h * scale)
        return image.resize((new_w, new_h), Image.LANCZOS)

    def image_to_base64(self, image: Image.Image) -> str:
        img = self._ensure_max_size(image)
        buffer = io.BytesIO()
        img.save(buffer, format="JPEG", quality=self.quality)
        return base64.b64encode(buffer.getvalue()).decode("utf-8")

    def image_to_data_url(self, image: Image.Image) -> str:
        b64 = self.image_to_base64(image)
        return f"data:image/jpeg;base64,{b64}"

    def find_image_on_screen(self, template: Image.Image, screen: Image.Image, threshold: float = 0.8):
        screen_cv = np.array(screen)
        screen_cv = cv2.cvtColor(screen_cv, cv2.COLOR_RGB2BGR)
        template_cv = np.array(template)
        template_cv = cv2.cvtColor(template_cv, cv2.COLOR_RGB2BGR)

        result = cv2.matchTemplate(screen_cv, template_cv, cv2.TM_CCOEFF_NORMED)
        _, max_val, _, max_loc = cv2.minMaxLoc(result)

        if max_val >= threshold:
            h, w = template_cv.shape[:2]
            center_x = max_loc[0] + w // 2
            center_y = max_loc[1] + h // 2
            return center_x, center_y, max_val
        return None

    def highlight_regions(self, image: Image.Image, regions: list) -> Image.Image:
        img_cv = np.array(image)
        img_cv = cv2.cvtColor(img_cv, cv2.COLOR_RGB2BGR)
        for x, y, w, h, label in regions:
            cv2.rectangle(img_cv, (x, y), (x + w, y + h), (0, 255, 0), 2)
            if label:
                cv2.putText(img_cv, label, (x, y - 5),
                            cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 1)
        return Image.fromarray(cv2.cvtColor(img_cv, cv2.COLOR_BGR2RGB))
