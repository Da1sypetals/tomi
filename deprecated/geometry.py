from pydantic.dataclasses import dataclass


@dataclass
class Resolution:
    x: int
    y: int


@dataclass
class Rect:
    xmin: float
    xmax: float
    ymin: float
    ymax: float


class Rescale:
    def __init__(self, resolution: Resolution, rect: Rect):
        self.resolution = resolution
        self.rect = rect

    def rescale(self, x, y):
        """
        Given (x, y) in the coord system same as self.rect
        Return coordinate in resolution system.
        """
        x_img = (x - self.rect.xmin) / (self.rect.xmax - self.rect.xmin) * self.resolution.x
        y_img = (y - self.rect.ymin) / (self.rect.ymax - self.rect.ymin) * self.resolution.y

        return x_img, y_img
