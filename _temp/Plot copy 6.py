import pandas as pd
import plotly.express as px
import plotly.graph_objects as go

# Load the new dataset
df = pd.read_csv('_temp/source.csv')

# Drop rows where key values are missing
df_cleaned = df.dropna(subset=['EquivalentChainLength', 'OnsetTemperature', 'TemperatureStep', 'FattyAcid'])

# Create a copy for modifications
df_plot = df_cleaned.copy()

# Convert TemperatureStep to a categorical variable for symbols
df_plot['TemperatureStep_cat'] = df_plot['TemperatureStep'].astype(str)

# --- Create the base scatter plot ---
# We will add lines to this figure later
fig = px.scatter_3d(
    df_plot,
    x='EquivalentChainLength',
    y='OnsetTemperature',
    z='TemperatureStep',
    color='OnsetTemperature',
    symbol='TemperatureStep_cat',
    color_continuous_scale=px.colors.sequential.Viridis,
    title='3D-график с фильтром по Fatty Acid',
    labels={
        'EquivalentChainLength': 'Equivalent Chain Length',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step',
        'TemperatureStep_cat': 'Temp. Step'
    },
    hover_data={
        'FattyAcid': True,
        'EquivalentChainLength': ':.2f',
        'TemperatureStep_cat': False # Hide the categorical version from hover
    }
)

# --- Add line traces for each FattyAcid ---
# This allows us to control their visibility independently
unique_fatty_acids = df_plot['FattyAcid'].unique()
lines_to_add = []

for acid in unique_fatty_acids:
    acid_df = df_plot[df_plot['FattyAcid'] == acid].sort_values(by=['TemperatureStep', 'OnsetTemperature'])
    
    line_trace = go.Scatter3d(
        x=acid_df['EquivalentChainLength'],
        y=acid_df['OnsetTemperature'],
        z=acid_df['TemperatureStep'],
        mode='lines',
        line=dict(width=2.5, color='grey'),
        name=acid, # Name for hover, but won't show in legend by default
        showlegend=False,
        hoverinfo='none' # Disable hover info for the lines themselves
    )
    lines_to_add.append(line_trace)

# Add all line traces to the figure
fig.add_traces(lines_to_add)


# --- Create the dropdown menu (updatemenu) ---
buttons = []

# Button to show all lines
buttons.append(dict(
    method='restyle',
    label='Показать все',
    args=[{'visible': [True] * (len(fig.data))}] # Make all traces visible
))

# Create a button for each FattyAcid
for i, acid in enumerate(unique_fatty_acids):
    # Create a visibility mask. True for scatter points and the one selected line.
    visibility_mask = [True] * (len(fig.data) - len(unique_fatty_acids)) + [False] * len(unique_fatty_acids)
    visibility_mask[len(fig.data) - len(unique_fatty_acids) + i] = True
    
    buttons.append(dict(
        method='restyle',
        label=acid,
        args=[{'visible': visibility_mask}]
    ))

# Add the dropdown to the layout
fig.update_layout(
    updatemenus=[
        dict(
            active=0,
            buttons=buttons,
            direction="down",
            pad={"r": 10, "t": 10},
            showactive=True,
            x=0.01,
            xanchor="left",
            y=0.99,
            yanchor="top"
        )
    ],
    annotations=[
        dict(text="Fatty Acid:", showarrow=False,
             x=0, y=1.05, yref="paper", align="left")
    ]
)

# Show the figure
fig.show()
