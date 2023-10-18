import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

data = pd.read_csv('out.csv')

grid_data = data.pivot(index='discount', columns='learning_rate', values='score')

plt.figure(figsize=(10, 6))  # Adjust the figure size as needed
sns.heatmap(grid_data, cmap='viridis', annot=True, fmt=".4f", cbar_kws={'label': 'Score'})
plt.title('2D Grid of Scores')
plt.xlabel('Learning Rate')
plt.ylabel('Discount')
plt.show()

