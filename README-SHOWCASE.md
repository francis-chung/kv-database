# KV-Database Showcase Website

This is a beautiful, interactive showcase website for the KV-Database Rust project that highlights:

## Features Showcased

1. **Product Introduction** - Clear explanation of what KV-Database is and its unique value proposition
2. **Feature Highlights** - Visual cards showcasing:
   - Blazing fast performance (Rust + Tokio)
   - Full key-value functionality 
   - Sorted set operations
   - Data durability (WAL + snapshots)
3. **Complete Command Reference** - All available Redis-like commands with descriptions:
   - Key-Value: GET, SET, DEL, EXISTS, DBSIZE, CLEAR
   - Sorted Sets: ZADD, ZSCORE, ZREM, ZRANGE
4. **Interactive Demo** - Live, working simulation of the database that lets users:
   - Select commands from dropdown
   - Enter parameters and see real-time results
   - Experience the exact same output format as the real TCP server
5. **Usage Instructions** - Clear steps to run the actual database locally
6. **Deployment Guidance** - Information about deploying to Vercel (for frontend/demo) or wrapping in an API

## Technical Implementation

- **Pure HTML/CSS/Vanilla JS** - No frameworks needed, easy to deploy anywhere
- **Responsive Design** - Works on mobile, tablet, and desktop
- **Modern UI** - Clean, professional aesthetic with gradients, shadows, and animations
- **Interactive Simulation** - JavaScript model that mimics the real database behavior
- **Terminal-Style Interface** - Authentic look and feel matching the actual TCP server output

## How to Use

1. Open `kv-database-showcase.html` in any web browser
2. Use the interactive demo to try commands
3. Follow the usage instructions to run the actual Rust database
4. Deploy to Vercel or any static hosting service for the showcase

## Vercel Deployment

While the actual KV-Database is a TCP server best deployed on traditional servers or cloud VMs, this showcase HTML file can be deployed directly to Vercel:

```bash
vercel kv-database-showcase.html
```

For production use, consider deploying:
1. The actual Rust database to a cloud VM or container service
2. This showcase to Vercel/Netlify as a demonstration frontend
3. Optionally, create a lightweight API wrapper (Node.js/Go/Python) on Vercel that connects to your TCP database

## Customization

The showcase is easily customizable:
- Modify the colors in the CSS variables
- Add more features or commands as they're implemented
- Update the tech stack as dependencies change
- Enhance the interactive demo with more sophisticated simulations

---

Built to showcase the elegance and power of this Rust-based key-value database with sorted set functionality.