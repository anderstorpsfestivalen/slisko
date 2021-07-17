from apa102_pi.driver import apa102
strip = apa102.APA102(num_led=132, mosi=10, sclk=11, order='rbg')
strip.set_global_brightness(31)
strip.clear_strip()