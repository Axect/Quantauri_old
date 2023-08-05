import matplotlib.pyplot as plt
import pandas as pd
import FinanceDataReader as fdr
import os
import argparse

# Parse arguments
parser = argparse.ArgumentParser()
parser.add_argument('--code', type=str, default='005930')
args = parser.parse_args()

# Get data
today = pd.Timestamp.today().strftime('%Y-%m-%d')
df = fdr.DataReader(args.code, '2000-01-01', today)

# Extraact date & Close
dg = pd.DataFrame({
    'date': df.index.strftime('%Y-%m-%d').to_list(),
    'close': df['Close'].to_list(),
})

# Save data to "data/{code}/close.parquet"
# if folder "data/{code}" does not exist, create it
if not os.path.exists(f"data/{args.code}"):
    os.makedirs(f"data/{args.code}")
dg.to_parquet(f"data/{args.code}/close.parquet")
