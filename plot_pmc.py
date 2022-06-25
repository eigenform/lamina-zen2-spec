#!/usr/bin/python3

import matplotlib
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

import json
from sys import argv

if len(argv) < 4:
    print("usage: plotpmc.py <graph title> <input file> <output file>")
    exit()

title = argv[1]
ifile = argv[2]
ofile = argv[3]

matplotlib.rcParams['font.family'] = ['monospace']
colors = [ "red", "orange", "green", "blue", "indigo", "violet" ]
#colors = [ 
#        "xkcd:red orange",
#        "xkcd:neon purple",
#        "xkcd:periwinkle",
#        "xkcd:green",
#        "xkcd:salmon",
#        "xkcd:cerulean",
#    ]


data = []
with open(ifile, "r") as f:
    # One event per line
    for line in f.readlines():
        x = line.strip().split("|")
        desc = x[0]
        arr  = x[1]
        d    = json.loads(arr)
        data.append((desc, [i for i in range(0, len(d))], d))

fig, axs = plt.subplots(len(data),1, sharex=True, sharey=True)

for (idx, ax) in enumerate(axs):
    ax.set_title(data[idx][0], loc='left')
    #ax.set_ylim(bottom=0, top=max(data[idx][2]))
    ax.yaxis.set_major_locator(MaxNLocator(integer=True))
    #ax.set_yticks(range(0, max(data[idx][2])))
    ax.plot(data[idx][1], data[idx][2], color=colors[idx])
    ax.set_ylim(ymin=0)

fig.set_dpi(200)
fig.suptitle(title)
plt.subplots_adjust(hspace=0.6)
fig.supylabel("Number of events")
fig.supxlabel("Time (sequential test number)")
plt.tight_layout()
#fig.align_labels()

plt.savefig(ofile, dpi=200)
print("Wrote to {}".format(ofile))

plt.show()
