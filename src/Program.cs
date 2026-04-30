using Youtube.Console.Commands;
using Spectre.Console.Cli;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("youtube");

    config.AddCommand<SearchCommand>("search")
        .WithDescription("Search YouTube for videos, shorts, and streams");

    config.AddCommand<ScrapeCommand>("scrape")
        .WithDescription("Scrape a YouTube URL (video, channel, playlist, or hashtag)");
});

return app.Run(args);
