import numpy as np
from matplotlib import pyplot as plt

stats = np.loadtxt(open('stats.txt'), dtype=int)
mean, var = stats[:,1], stats[:,2]
