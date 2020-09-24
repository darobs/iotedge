// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Util.Metrics
{
    using System.Globalization;
    using Microsoft.AspNetCore.Hosting;
    using Microsoft.Extensions.Logging;
    using Prometheus;

    using IPAddress = System.Net.IPAddress;

    public class MetricsListener : IMetricsListener
    {
        const string MetricsUrlPrefixFormat = "http://{0}:{1}/{2}/";

        readonly KestrelMetricServer metricServer;
        readonly IMetricsProvider metricsProvider;
        readonly MetricsListenerConfig listenerConfig;

        ILogger logger;

        public MetricsListener(MetricsListenerConfig listenerConfig, IMetricsProvider metricsProvider)
        {
            this.listenerConfig = Preconditions.CheckNotNull(listenerConfig, nameof(listenerConfig));
            string url = GetMetricsListenerUrlPrefix(listenerConfig);
            this.metricServer = new KestrelMetricServer(port: listenerConfig.Port);
            this.metricsProvider = Preconditions.CheckNotNull(metricsProvider, nameof(metricsProvider));
        }

        public void Start(ILogger logger)
        {
            this.logger = logger;
            this.logger?.LogInformation($"Starting metrics listener on {this.listenerConfig}");
            this.metricServer.Start(); // RunAsync() and save Task.
        }

        public void Dispose()
        {
            this.logger?.LogInformation("Stopping metrics listener");
            this.metricServer.Dispose();
        }

        static string GetMetricsListenerUrlPrefix(MetricsListenerConfig listenerConfig)
            => string.Format(CultureInfo.InvariantCulture, MetricsUrlPrefixFormat, listenerConfig.Host, listenerConfig.Port, listenerConfig.Suffix.Trim('/', ' '));
    }
}
