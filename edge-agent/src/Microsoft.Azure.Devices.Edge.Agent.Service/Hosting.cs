// Copyright (c) Microsoft. All rights reserved.
namespace Microsoft.Azure.Devices.Edge.Agent.Service
{
    using System.Collections.Generic;
    using System.Net;
    using System.Net.Security;
    using System.Net.Sockets;
    using System.Security.Authentication;
    using System.Security.Cryptography.X509Certificates;
    using Autofac;
    using Microsoft.AspNetCore.Hosting;
    using Microsoft.AspNetCore.Server.Kestrel.Https;
    using Microsoft.Azure.Devices.Edge.Util;
    using Microsoft.Extensions.Configuration;
    using Microsoft.Extensions.DependencyInjection;

    public class Hosting
    {
        Hosting(IWebHost webHost, IContainer container)
        {
            this.WebHost = webHost;
            this.Container = container;
        }

        public IContainer Container { get; }

        public IWebHost WebHost { get; }

        public static Hosting Initialize(IConfigurationRoot configuration)
        {

            int metricsPort = configuration.GetValue("httpSettings:metrics_port", 9600);
            IWebHostBuilder webHostBuilder = new WebHostBuilder()
                .UseKestrel(
                    options =>
                    {
                        options.Listen(
                            !Socket.OSSupportsIPv6 ? IPAddress.Any : IPAddress.IPv6Any,
                            mainPort,
                            listenOptions =>
                            {
                                listenOptions.UseHttps(connectionAdapterOptions);
                            });

                        options.Listen(!Socket.OSSupportsIPv6 ? IPAddress.Any : IPAddress.IPv6Any, metricsPort);
                    })
                .UseSockets()
                .ConfigureServices(
                    serviceCollection =>
                    {
                        serviceCollection.AddSingleton(configuration);
                    })
                .UseStartup<Startup>();
            IWebHost webHost = webHostBuilder.Build();
            IContainer container = webHost.Services.GetService(typeof(IStartup)) is Startup startup ? startup.Container : null;
            return new Hosting(webHost, container);
        }

    }
}
