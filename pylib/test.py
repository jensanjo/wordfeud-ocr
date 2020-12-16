from pywordfeud_ocr import recognize_screenshot
screenshot_filename = "../lib/screenshots/screenshot_blank_tile.png"
res = recognize_screenshot(screenshot_filename)
print("State:\n{}".format('\n'.join(res['state_ocr'])))
print("Rack: \"{}\"".format(res['rack_ocr']))
print("Board:\n{}".format('\n'.join(res['board_ocr'])))
print("Board area: {}".format(res['board_area']))
print("Rack area: {}".format(res['rack_area']))
