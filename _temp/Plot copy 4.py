import pandas as pd
import plotly.express as px

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['ECL'] % 1 != 0].copy()

# --- Key Change: Convert BOTH columns to string type for categorical legends ---
df_filtered['TemperatureStep'] = df_filtered['TemperatureStep'].astype(str)
df_filtered['OnsetTemperature'] = df_filtered['OnsetTemperature'].astype(str)


# Create the interactive 3D scatter plot
fig = px.scatter_3d(
    df_filtered,
    x='ECL',
    y='OnsetTemperature',
    z='TemperatureStep',
    color='OnsetTemperature',  # Now uses the string version for a categorical legend
    symbol='TemperatureStep',
    title='Интерактивный 3D-график с кликабельными легендами',
    labels={
        'ECL': 'ECL',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step'
    },
    hover_data=df_cleaned.columns # Show original (numeric) data on hover
)

# Update layout for better legend placement
fig.update_layout(
    margin=dict(l=0, r=0, b=0, t=40),
    legend=dict(
        title="Фильтры",
        yanchor="top",
        y=0.99,
        xanchor="left",
        x=0.01
    )
)

# Show the figure (this will be interactive in your environment)
fig.show()
