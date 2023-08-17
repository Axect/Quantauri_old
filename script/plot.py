import pandas as pd
import matplotlib.pyplot as plt
import scienceplots
import datetime as dt
import numpy as np
import os
import argparse

# Parse arguments
parser = argparse.ArgumentParser(description='Plotting')
parser.add_argument('--code', type=str, default='005930', help='Stock code')
#parser.add_argument('--start', type=str, default='2010-01-01', help='Start date')
#parser.add_argument('--end', type=str, default=dt.datetime.today().strftime('%Y-%m-%d'), help='End date')
args = parser.parse_args()

# Import parquet file
dg = pd.read_parquet(f'data/{args.code}/alpha.parquet')

# Plot directory
# If not exist, create directory
directory = f'plot/{args.code}/'
if not os.path.exists(directory):
    os.makedirs(directory)

# Prepare Data to Plot
date        = np.array([dt.datetime.strptime(x, '%Y-%m-%d') for x in dg.date], dtype=np.datetime64)
close       = dg["close"].to_numpy()
high        = dg["high"].to_numpy()
low         = dg["low"].to_numpy()
mean        = dg["mean"].to_numpy()
std_dev     = dg["std_dev"].to_numpy()
alpha       = dg["alpha"].to_numpy()
calpha      = dg["calpha"].to_numpy()
calpha2     = dg["calpha2"].to_numpy()
buy_sell    = dg["buy_sell"].to_numpy()

price_mean = np.mean(close)
price_min_diff = price_mean * 0.05

# Obtain today
today = np.datetime64('today')
xlim = (today-np.timedelta64(365, 'D'), today)
idx = np.where((date >= xlim[0]) & (date <= xlim[1]))[0]

# Set close ylim to be 90% of min and 110% of max in xlim
close_min = np.min(low[idx])
close_max = np.max(high[idx])
ylim = (close_min*0.99, close_max*1.01)

# Set calpha ylim to be 90% of min and 110% of max in xlim
calpha_min = np.min(calpha[idx])
calpha_max = np.max(calpha[idx])
ylim_calpha = (calpha_min - abs(calpha_min)*0.1, calpha_max + abs(calpha_max)*0.1)

idx_buy     = np.where(buy_sell == 1)[0] 
idx_sell    = np.where(buy_sell == -1)[0]
print(f"Buy count: {len(idx_buy)}, Sell count: {len(idx_sell)}")

# Compute profit for each buy-sell pair
profit = np.zeros(len(date))
for i in range(len(idx_sell)):
    profit[idx_sell[i]] = close[idx_sell[i]] - close[idx_buy[i]]
ylim_profit = (np.min(profit[idx]).min(0) - price_min_diff, np.max(profit[idx]).max(0) + price_min_diff)

# Cumulate Profit starting from xlim[0]
cum_profit = np.cumsum(profit)
cum_profit = cum_profit - cum_profit[idx[0]]
ylim_cprofit = (np.min(cum_profit[idx]).min(0) - price_min_diff, np.max(cum_profit[idx]).max(0) + price_min_diff)

xlim = (xlim[0], xlim[1] + np.timedelta64(5, 'D'))

# Plot params
pparam = dict(
    xlabel = r'Date',
    ylabel = r'Value',
    xscale = 'linear',
    yscale = 'linear',
    xlim = xlim,
    #ylim = (0, 1),
)

# Plot
with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.set_ylim(ylim)
    ax.plot(date, close, 'k')
    ax.plot(date, mean, 'g--', alpha=0.7, label=r'Moving Average ($t=5$)')
    ax.fill_between(date, mean-3*std_dev, mean+3*std_dev, color='gray', alpha=0.5, label=r'$3\sigma$')
    ax.plot(date[idx_buy], close[idx_buy], 'bo', label='Buy')
    ax.plot(date[idx_sell], close[idx_sell], 'ro', label='Sell')
    for i in range(len(idx_buy)):
        ax.annotate(
            f'{close[idx_buy[i]]:0.0f}',
            (date[idx_buy[i]], close[idx_buy[i]]),
            xytext=(date[idx_buy[i]], close[idx_buy[i]]-price_min_diff),
            fontsize=4,
            bbox=dict(boxstyle="round", fc="w", ec="0.5", alpha=0.5)
        )
    for i in range(len(idx_sell)):
        ax.annotate(
            f'{close[idx_sell[i]]:0.0f}',
            (date[idx_sell[i]], close[idx_sell[i]]),
            xytext=(date[idx_sell[i]], close[idx_sell[i]]+price_min_diff),
            fontsize=4,
            bbox=dict(boxstyle="round", fc="w", ec="0.5", alpha=0.5)
        )
    ax.grid()
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig(directory + 'close.png', dpi=600, bbox_inches='tight')

with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.set_ylim(ylim_calpha)
    ax.plot(date, calpha, 'k', label='Cumulative Alpha (T=5)')
    ax.plot(date, calpha2, 'g--', alpha=0.7, label='Cumulative Alpha (T=20)')
    ax.plot(date[idx_buy], calpha[idx_buy], 'bo', label='Buy')
    ax.plot(date[idx_sell], calpha[idx_sell], 'ro', label='Sell')
    ax.axhline(y=0, color='gray', linestyle='--', alpha=0.5)
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig(directory + 'calpha.png', dpi=600, bbox_inches='tight')

# Profit plot
with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.set_ylim(ylim_profit)
    # Bar plot (profit>0: Red, profit<0: Blue)
    ax.bar(date, profit, width=np.datetime64(5, 'D'), color=np.where(profit>0, 'r', 'b'), label='Profit')
    #ax.bar(date[idx_buy], profit[idx_buy], width=np.datetime64(5, 'D'), color='r', label='Buy')
    #ax.bar(date[idx_sell], profit[idx_sell], width=np.datetime64(5, 'D'), color='b', label='Sell')
    ax.axhline(y=0, color='r', linestyle='--')
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig(directory + 'profit.png', dpi=600, bbox_inches='tight')

# Cumulative Profit plot
with plt.style.context(["science", "nature"]):
    fig, ax = plt.subplots()
    ax.autoscale(tight=True)
    ax.set(**pparam)
    ax.set_ylim(ylim_cprofit)
    ax.plot(date, cum_profit, label='Cumulative Profit')
    ax.axhline(y=0, color='r', linestyle='--')
    ax.legend()
    fig.autofmt_xdate()
    fig.savefig(directory + 'cprofit.png', dpi=600, bbox_inches='tight')
