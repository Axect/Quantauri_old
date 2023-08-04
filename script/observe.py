import matplotlib.pyplot as plt
import pandas as pd
import FinanceDataReader as fdr

# KB
df = fdr.DataReader('105560', '2000-01-01', '2023-08-04')

# Extraact date & Close
dg = pd.DataFrame({
    'date': df.index.strftime('%Y-%m-%d').to_list(),
    'close': df['Close'].to_list(),
})

dg.to_parquet("data/KB.parquet")
