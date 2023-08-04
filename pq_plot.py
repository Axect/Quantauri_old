import pandas as pd
import matplotlib.pyplot as plt
import scienceplots
import datetime as dt
import numpy as np

# Import parquet file
dg = pd.read_parquet('data/KB_alpha.parquet')

# Prepare Data to Plot
date = np.array([dt.datetime.strptime(x, '%Y-%m-%d') for x in dg.date])
close   = dg["close"].to_numpy()
mean    = dg["mean"].to_numpy()
std_dev = dg["std_dev"].to_numpy()
dev     = dg["dev"].to_numpy()
alpha   = dg["alpha"].to_numpy()
calpha  = dg["calpha"].to_numpy()

# Find sign change in calpha
# buy: calpha: + -> -
idx_buy = np.where(np.diff(np.sign(calpha)) == -2)[0]
# sell: calpha: - -> +
idx_sell = np.where(np.diff(np.sign(calpha)) == 2)[0]
print(f"Buy count: {len(idx_buy)}, Sell count: {len(idx_sell)}")

# Compute profit for each buy-sell pair
profit = np.zeros(len(idx_sell))
for i in range(len(idx_sell)):
    profit[i] = close[idx_sell[i]] - close[idx_buy[i]]

# Plot params
pparam = dict(
    xlabel = r'Date',
    ylabel = r'Value',
    xscale = 'linear',
    yscale = 'linear',
    xlim = (dt.datetime(2022, 1, 1), dt.datetime(2023, 8, 4)),
    #ylim = (0, 1),
)

# Plot
with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.set_ylim(42000, 68000)
    ax.plot(date, close)
    ax.plot(date, mean, label=r'Moving Average ($t=5$)')
    ax.fill_between(date, mean-2*std_dev, mean+2*std_dev, alpha=0.2, label=r'$\mu \pm 2\sigma$')
    ax.plot(date[idx_buy], close[idx_buy], 'bo', label='Buy')
    ax.plot(date[idx_sell], close[idx_sell], 'ro', label='Sell')
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig('plot/KB_close.png', dpi=600, bbox_inches='tight')

with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.plot(date, alpha, label='Alpha')
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig('plot/KB_alpha.png', dpi=600, bbox_inches='tight')

with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.plot(date, calpha, label='Cumulative Alpha')
    ax.plot(date[idx_buy], calpha[idx_buy], 'bo', label='Buy')
    ax.plot(date[idx_sell], calpha[idx_sell], 'ro', label='Sell')
    ax.axhline(y=0, color='gray', linestyle='--', alpha=0.5)
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig('plot/KB_calpha.png', dpi=600, bbox_inches='tight')

# Profit plot
with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.plot(date[idx_sell], profit, label='Profit')
    ax.axhline(y=0, color='r', linestyle='--')
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig('plot/KB_profit.png', dpi=600, bbox_inches='tight')
