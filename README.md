# rusty-tracker #

A web interface and tracker for a bittorrent downloads using rust.

## Design ##

- A screen to search, visualize and download torrents
- A user system to allow metainfo file uploads
- The bittorrent tracker which coordinates the transfer of files between users
- This will be initially a private tracker

- Will be backed with sqlite to be simple as possible
- Will use sqlx as the sql lib
- Will use handlebars as the templating lib
- Will use axum as web framework

### My line of though on this ###

- A private tracker website basically is a listing of metainfo files with their associated metada behind a authorization/authentication system that control their visualization from the public internet
- And also has the bittorrent tracker action that allow users download the respective files shared with the bittorrent protocol

## A overview list of things to be done ##

- Create the interface with axum, handlebars
    - The file listing
    - the torrent details page
    - profile page for the user
    - general stats about the tracker
    - login page
    - register page
- Add the auth system with user tiers
    - Simple user (will be allowed to download and upload torrents)
    - Admin user (will be able to moderate (reject) torrents and manage general aspects of the tracker)
- Implement the bittorrent tracker protocol to enable the file sharing