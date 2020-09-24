// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Util.Metrics
{
    using System.Globalization;
    using System.Threading;
    using System.Threading.Tasks;
    using global::Prometheus;
    using Microsoft.AspNetCore.Builder;
    using Microsoft.AspNetCore.Hosting;
    using Microsoft.Extensions.DependencyInjection;
    using Microsoft.Extensions.Hosting;
    using Microsoft.Extensions.Logging;

    using Host = Extensions.Hosting.Host;
    using IPAddress = System.Net.IPAddress;

    public class MetricsListener : IMetricsListener
    {
        const string MetricsUrlPrefixFormat = "http://{0}:{1}/{2}/";
        readonly CancellationTokenSource cts = new CancellationTokenSource();
        readonly IWebHost metricServer;
        readonly IMetricsProvider metricsProvider;
        readonly MetricsListenerConfig listenerConfig;

        ILogger logger;
        Task servicingTask;

        public MetricsListener(MetricsListenerConfig listenerConfig, IMetricsProvider metricsProvider)
        {
            this.listenerConfig = Preconditions.CheckNotNull(listenerConfig, nameof(listenerConfig));
            string url = GetMetricsListenerUrlPrefix(listenerConfig);
            this.metricServer = CreateHostBuilder().Build();
            this.metricsProvider = Preconditions.CheckNotNull(metricsProvider, nameof(metricsProvider));
        }

        public void Start(ILogger logger)
        {
            this.logger = logger;
            this.logger?.LogInformation($"Starting metrics listener on {this.listenerConfig}");
            this.servicingTask = this.metricServer.RunAsync(this.cts.Token); // RunAsync() and save Task.
        }

        public void Dispose()
        {
            this.logger?.LogInformation("Stopping metrics listener");
            this.cts.Cancel();
            this.servicingTask.Wait();
            this.metricServer.Dispose();
        }

        static string GetMetricsListenerUrlPrefix(MetricsListenerConfig listenerConfig)
            => string.Format(CultureInfo.InvariantCulture, MetricsUrlPrefixFormat, listenerConfig.Host, listenerConfig.Port, listenerConfig.Suffix.Trim('/', ' '));

        static IHostBuilder CreateHostBuilder() =>
            Host.CreateDefaultBuilder()
                .ConfigureWebHostDefaults(webBuilder =>
                {
                    webBuilder.UseStartup<Startup>();
                });
    }

    public class Startup
    {
        // This method gets called by the runtime. Use this method to add services to the container.
        // For more information on how to configure your application, visit https://go.microsoft.com/fwlink/?LinkID=398940
        public void ConfigureServices(IServiceCollection services)
        {
        }

        // This method gets called by the runtime. Use this method to configure the HTTP request pipeline.
        public void Configure(IApplicationBuilder app, IWebHostEnvironment env)
        {
            app.UseRouting();
            app.UseHttpMetrics();
        }
    }

}
