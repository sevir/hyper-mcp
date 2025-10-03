# Google Custom Search API Requirements

## Summary

The **Custom Search Engine ID (`cx` parameter) is MANDATORY** for the Google Custom Search JSON API. The current implementation in the google-search plugin is correct in requiring both `GOOGLE_API_KEY` and `GOOGLE_SEARCH_ENGINE_ID`.

## Why it's Required

The `cx` parameter specifies which Programmable Search Engine (formerly Custom Search Engine, CSE) you want to use for your queries. Without it, the API cannot determine which search engine configuration to apply and will not return results.

## How to Obtain the Search Engine ID

### Step-by-Step Guide

1. **Go to Google Programmable Search Engine**
   - Visit [programmablesearchengine.google.com](https://programmablesearchengine.google.com)

2. **Create a New Search Engine**
   - Click "Get started"
   - Enter the websites you want to search, or select the option to search the entire web
   - Give your search engine a name
   - Click "Create"

3. **Access the Control Panel**
   - After creation, you'll be taken to the Control Panel for your new search engine
   - Customize settings such as sites to search, language and region, image search and SafeSearch options

4. **Find Your Search Engine ID (cx)**
   - In the Control Panel, look for the "Search Engine ID"
   - It's typically displayed on the main dashboard or under "Details" or "Basics"
   - The cx will look like: `012345678901234567890:abcde_fghij`

5. **Get an API Key (if needed)**
   - Go to the [Google Cloud Console](https://console.developers.google.com/)
   - Create or select a project
   - Enable the "Custom Search API"
   - Go to "APIs & Services > Credentials" and create an "API key"

## Current Implementation Status

The google-search plugin correctly requires both:
- `GOOGLE_API_KEY` (Required)
- `GOOGLE_SEARCH_ENGINE_ID` (Required)

Both parameters should remain mandatory as per Google's API requirements.

## Documentation

The plugin's README.md already contains comprehensive documentation about:
- Prerequisites for setting up Google Custom Search
- How to create a Custom Search Engine
- Configuration requirements
- Rate limits and quotas (100 queries/day free, 10,000 paid)

## Conclusion

**No changes needed** - the plugin implementation is correct. Both environment variables are mandatory for the Google Custom Search JSON API to function properly.
